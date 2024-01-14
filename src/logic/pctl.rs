use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::process::exit;
use log::error;
use pest::iterators::{Pair};
use pest::Parser;
use pest_derive::Parser;
use petgraph::graph::{Edges, NodeIndex};
use petgraph::{Incoming, Outgoing};
use petgraph::visit::EdgeRef;
use crate::logic::{Formula, LogicImpl};
use crate::mcsp::PctlInfo;
use crate::utils::common::Comp;
use crate::input_graph::{MDP, Node};

#[derive(Parser)]
#[grammar = "logic/pctl.pest"]
struct PctlPestParser;

pub struct PctlImpl;

impl PctlImpl {
    fn parse_state(pair: &Pair<Rule>) -> Box<dyn StatePhi>{
        let inner_rules = pair.clone().into_inner().collect::<Vec<Pair<Rule>>>();
        match pair.as_rule() {
            Rule::Phi_and => {
                let left_rule = Self::parse_state(inner_rules.first().unwrap());
                let right_rule = Self::parse_state(inner_rules.get(1).unwrap());
                Box::new(AndPhi {
                    left_phi: left_rule,
                    right_phi: right_rule,
                })
            }
            Rule::Phi_not => {
                let inner_phi = Self::parse_state(inner_rules.first().unwrap());
                Box::new(NotPhi { phi: inner_phi })
            }
            Rule::ap => Box::new(AP {
                value: pair.as_str().into(),
            }),
            Rule::r#true => Box::new(True {}),
            Rule::prob => {
                let inner_phi = Self::parse_path(inner_rules.first().unwrap());
                let comp_char: &str = inner_rules.get(1).unwrap().as_str();
                let comp: Comp = match comp_char {
                    "<" => Comp::Less,
                    "<=" => Comp::Leq,
                    ">" => Comp::Greater,
                    ">=" => Comp::Geq,
                    _ => {
                        error!(
                        "Syntax error! \"{}\" is not a valid comparison character. Terminating...",
                        comp_char
                    );
                        exit(0);
                    }
                };
                let prob_str: &str = inner_rules.get(2).unwrap().as_str();
                match prob_str.parse::<f64>() {
                    Ok(prob) => Box::new(Prob {
                        phi: inner_phi,
                        comp,
                        probability: prob,
                    }),
                    Err(_) => {
                        error!("\"{}\" is not a valid float! Terminating...", prob_str);
                        exit(0);
                    }
                }
            }
            _ => panic!("Rule is invalid or should have been processed by parent!"),
        }
    }

    fn parse_path(pair: &Pair<Rule>) -> Box<dyn PathPhi>{
        let inner_rules = pair.clone().into_inner().collect::<Vec<Pair<Rule>>>();
        match pair.as_rule() {
            Rule::phi_next => {
                let inner_phi = Self::parse_state(inner_rules.first().unwrap());
                Box::new(Next { phi: inner_phi })
            }
            Rule::phi_until => {
                let left_phi = Self::parse_state(inner_rules.first().unwrap());
                let right_phi = Self::parse_state(inner_rules.get(1).unwrap());
                Box::new(Until {
                    prev: left_phi,
                    until: right_phi,
                })
            }
            _ => panic!(),
        }
    }
}

impl LogicImpl for PctlImpl {
    fn parse(&self, content: &str) -> Box<dyn Formula>{
        let phi_content = match self.find_formula(content) {
            None => panic!("Formula must contain 'PHI' exactly once!"),
            Some(c) => c
        };
        let pairs: Vec<_> = PctlPestParser::parse(Rule::Main, &phi_content).unwrap().collect();
        let pair = pairs.first().unwrap();
        let state_phi: Box<dyn StatePhi> = Self::parse_state(pair);
        Box::new(PctlFormula(state_phi))
    }
}

struct PctlFormula(Box<dyn StatePhi>);

impl Formula for PctlFormula {
    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        self.0.evaluate(pctl_info)
    }
}

pub trait StatePhi {
    fn fmt(&self) -> String;
    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex>;
}

impl Display for dyn StatePhi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fmt())
    }
}

pub trait PathPhi {
    fn fmt(&self) -> String;
    fn evaluate(
        &self,
        pctl_info: &PctlInfo,
        comp: &Comp,
        prob_bound: f64,
    ) -> HashSet<NodeIndex>;
}

impl Display for dyn PathPhi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fmt())
    }
}

pub struct True;

impl StatePhi for True {
    fn fmt(&self) -> String {
        "true".into()
    }

    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        pctl_info
            .reach_graph
            .node_indices()
            .filter(|n| pctl_info.reach_graph[*n].is_state())
            .collect()
    }
}

pub struct NotPhi {
    pub phi: Box<dyn StatePhi>,
}

impl StatePhi for NotPhi {
    fn fmt(&self) -> String {
        format!("¬ ({})", self.phi)
    }

    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        let phi_nodes = self.phi.evaluate(pctl_info);
        let all_nodes: HashSet<NodeIndex> = pctl_info
            .reach_graph
            .node_indices()
            .filter(|n| pctl_info.reach_graph[*n].is_state())
            .collect();
        all_nodes.difference(&phi_nodes).copied().collect()
    }
}

pub struct Prob {
    pub phi: Box<dyn PathPhi>,
    pub probability: f64,
    pub comp: Comp,
}

impl StatePhi for Prob {
    fn fmt(&self) -> String {
        format!("P(({}), {} {})", self.phi, self.comp, self.probability)
    }

    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        self.phi
            .evaluate(pctl_info, &self.comp, self.probability)
    }
}

pub struct AP {
    pub value: String,
}

impl StatePhi for AP {
    fn fmt(&self) -> String {
        self.value.clone()
    }

    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        pctl_info.ap_map[&self.value].clone()
    }
}

pub struct AndPhi {
    pub left_phi: Box<dyn StatePhi>,
    pub right_phi: Box<dyn StatePhi>,
}

impl StatePhi for AndPhi {
    fn fmt(&self) -> String {
        format!("({}) ∧ ({})", self.left_phi, self.right_phi)
    }

    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        let left_markings = self.left_phi.evaluate(pctl_info);
        let right_markings = self.right_phi.evaluate(pctl_info);
        left_markings
            .intersection(&right_markings)
            .copied()
            .collect()
    }
}


pub struct Next {
    pub phi: Box<dyn StatePhi>,
}

pub struct Until {
    pub prev: Box<dyn StatePhi>,
    pub until: Box<dyn StatePhi>,
}

impl PathPhi for Next {
    fn fmt(&self) -> String {
        format!("◯ ({})", self.phi)
    }

    fn evaluate<>(
        &self,
        pctl_info: &PctlInfo,
        comp: &Comp,
        #[allow(unused_variables)]
        prob_bound: f64,
    ) -> HashSet<NodeIndex> {
        let graph = &pctl_info.reach_graph;
        let phi_node_indices = self.phi.evaluate(pctl_info);

        // Create a hashmap which maps every action to a probability of satisfying phi
        let mut action_prob: HashMap<NodeIndex, f64> = HashMap::new();
        let mut state_prob: HashMap<NodeIndex, f64> = HashMap::new();
        for phi_node_index in phi_node_indices {
            let pre_actions = graph.neighbors_directed(phi_node_index, Incoming)
                .collect::<Vec<NodeIndex>>();
            for pre_action in pre_actions {
                let old_prob = action_prob.get(&pre_action).unwrap_or(&0.0);
                let prob = graph
                    .edges_connecting(pre_action, phi_node_index)
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap()
                    .weight();
                action_prob.insert(pre_action, old_prob + prob);
            }
        }

        // Create a hashmap which maps all states to the max or min (depending on comp) probability
        // of satisfying phi
        for action in action_prob.keys() {
            let pre_states = graph.neighbors_directed(*action, Incoming)
                .collect::<Vec<NodeIndex>>();
            for pre_state in pre_states {
                let old_prob = state_prob.get(&pre_state);
                let new_prob = match old_prob {
                    None => action_prob[action],
                    Some(p) => if comp.is_upper_bound() {
                        p.max(action_prob[action])
                    } else {
                        p.min(action_prob[action])
                    }
                };
                state_prob.insert(pre_state, new_prob);
            }
        }

        // Returns the hashmap containing the states for which the comparison holds
        pctl_info
            .reach_graph
            .node_indices()
            .filter(|n| pctl_info.reach_graph[*n].is_state())
            .filter(|n| comp.evaluate(state_prob.get(n).unwrap_or(&0.0), &prob_bound))
            .collect()
    }
}

#[allow(unused_variables)]
impl Until {
    fn w_op(
        &self,
        left_tsi: &HashSet<NodeIndex>,
        right_tsi: &HashSet<NodeIndex>,
        all_indices: &HashSet<NodeIndex>,
        graph: &MDP<NodeIndex>,
    ) -> HashSet<NodeIndex> {
        let mut new = all_indices.clone();
        loop {
            let not_new = all_indices.difference(&new);
            let tmp2: HashSet<NodeIndex> = not_new
                .flat_map(|i| graph.neighbors_directed(*i, Incoming))
                .flat_map(|i| graph.neighbors_directed(i, Incoming))
                .collect();
            let tmp3: HashSet<NodeIndex> = all_indices.difference(&tmp2).copied().collect();
            let tmp4: HashSet<NodeIndex> = left_tsi.intersection(&tmp3).copied().collect();
            let tmp5: HashSet<NodeIndex> = right_tsi.union(&tmp4).copied().collect();
            let tmp6: HashSet<NodeIndex> = new.intersection(&tmp5).copied().collect();
            if tmp6 == new {
                break;
            } else {
                new = tmp6.clone();
            };
        }
        new
    }

    fn u_op(
        &self,
        left_tsi: &HashSet<NodeIndex>,
        right_tsi: &HashSet<NodeIndex>,
        all: &HashSet<NodeIndex>,
        graph: &MDP<NodeIndex>,
    ) -> HashSet<NodeIndex> {
        let mut new = right_tsi.clone();
        let only_pre: HashSet<NodeIndex> = graph
            .node_indices()
            .flat_map(|i| graph.neighbors_directed(i, Incoming))
            .flat_map(|i| graph.neighbors_directed(i, Incoming))
            .collect();
        loop {
            let tmp1 = all.difference(&new);
            let tmp2: HashSet<NodeIndex> = tmp1
                .flat_map(|i| graph.neighbors_directed(*i, Incoming))
                .flat_map(|i| graph.neighbors_directed(i, Incoming))
                .collect();
            let tmp3: HashSet<NodeIndex> = all.difference(&tmp2).copied().collect();
            let tmp4: HashSet<NodeIndex> = left_tsi.intersection(&tmp3).copied().collect();
            let tmp5: HashSet<NodeIndex> = only_pre.intersection(&tmp4).copied().collect();
            let tmp6: HashSet<NodeIndex> = new.union(&tmp5).copied().collect();
            if tmp6 == new {
                break;
            } else {
                new = tmp6.clone();
            };
        }
        new
    }
}

#[allow(unused_variables)]
impl PathPhi for Until {
    fn fmt(&self) -> String {
        format!("({}) U ({})", self.prev, self.until)
    }

    fn evaluate(
        &self,
        pctl_info: &PctlInfo,
        comp: &Comp,
        prob_bound: f64,
    ) -> HashSet<NodeIndex> {
        let all: HashSet<_> = pctl_info
            .reach_graph
            .node_indices()
            .filter(|i| matches!(pctl_info.reach_graph[*i], Node::State(_)))
            .collect();

        let geq_zero = *comp == Comp::Geq && prob_bound == 0.0;
        let leq_one = *comp == Comp::Leq && prob_bound == 1.0;
        if geq_zero || leq_one {
            return all;
        }

        let left_phi = self.prev.evaluate(pctl_info);
        let right_phi = self.until.evaluate(pctl_info);

        // not E(phi_1 U phi_2) = A(not phi_2 W (not phi_1 and not phi_2))
        let not_left_phi: HashSet<NodeIndex> = all.difference(&left_phi).copied().collect();
        let not_right_phi: HashSet<NodeIndex> = all.difference(&right_phi).copied().collect();
        let not_left_and_not_right: HashSet<_> =
            not_left_phi.intersection(&not_right_phi).copied().collect();

        let left_tsi = not_right_phi;
        let right_tsi = not_left_and_not_right;

        let mut prob_map: HashMap<NodeIndex, f64> = HashMap::new();
        let s_0 = self.w_op(&left_tsi, &right_tsi, &all, &pctl_info.reach_graph);
        for index in &s_0 {
            prob_map.insert(*index, 0.0);
        }
        let s_1 = self.u_op(&left_phi, &right_phi, &all, &pctl_info.reach_graph);
        for index in &s_1 {
            prob_map.insert(*index, 1.0);
        }
        let s_q: HashSet<NodeIndex> = all
            .difference(&s_0.union(&s_1).copied().collect())
            .copied()
            .collect();
        for node_index in &s_q {
            prob_map.insert(*node_index, 0.0);
        }
        let graph = &pctl_info.reach_graph;
        loop {
            let mut max_error: f64 = 0.0;
            for node_index in &s_q {
                let actions = graph.neighbors_directed(*node_index, Outgoing);
                let mut prob: Option<f64> = None;
                for action in actions {
                    let edges: Edges<_, _> = graph.edges_directed(action, Outgoing);
                    let prob_to_sq: f64 = edges
                        .clone()
                        .filter(|i| s_q.contains(&i.target()))
                        .map(|i| i.weight() * prob_map.get(&i.target()).unwrap())
                        .sum();
                    let prob_to_s1: f64 = edges
                        .filter(|i| s_1.contains(&i.target()))
                        .map(|i| i.weight())
                        .sum();
                    prob = match prob {
                        None => Some(prob_to_sq + prob_to_s1),
                        Some(p) => if comp.is_upper_bound() {
                            Some(p.max(prob_to_sq + prob_to_s1))
                        } else {
                            Some(p.min(prob_to_sq + prob_to_s1))
                        }
                    }
                }
                max_error = max_error.max((prob_map[node_index] - prob.unwrap()).abs());
                prob_map.insert(*node_index, prob.unwrap());
            }
            if max_error < pctl_info.max_error {
                break;
            }
        }
        prob_map
            .into_iter()
            .filter(|(k, v)| comp.evaluate(*v, prob_bound))
            .map(|(k, v)| k)
            .collect()
    }
}