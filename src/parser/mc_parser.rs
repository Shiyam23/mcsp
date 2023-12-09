use std::{
    collections::{HashMap, HashSet},
    process::exit,
    str::FromStr,
};

use crate::mcsp::ModelCheckInfo;
use crate::parser::parser::Rule;
use log::error;
use pest::iterators::Pair;
use petgraph::graph::Edges;
use petgraph::visit::EdgeRef;
use petgraph::{graph::{DiGraph, NodeIndex}, Direction::Incoming, Outgoing};
use crate::pctl::{AP, Next, NotPhi, PathPhi, Prob, StatePhi, Until, True};
use crate::utils::common::Comp;

type Marking = Vec<usize>;
type Markings = Vec<Marking>;
type ApMap = HashMap<String, Markings>;

pub fn transform_ap_map(pair: &Pair<Rule>) -> ApMap {
    assert_eq!(pair.as_rule(), Rule::AP);
    let mut ap_map = ApMap::new();
    for ap_assign in pair.clone().into_inner() {
        let elements: Vec<Pair<Rule>> = ap_assign.into_inner().collect();
        match (elements.first(), elements.get(1)) {
            (Some(ap), Some(markings_rule)) => {
                let markings = markings_rule
                    .clone()
                    .into_inner()
                    .map(|marking| {
                        marking
                            .into_inner()
                            .map(|int_rule| int_rule.as_str().parse_or_quit("integer"))
                            .collect()
                    })
                    .collect();
                ap_map.insert(ap.as_str().into(), markings);
            }
            (_, _) => panic!(),
        }
    }
    ap_map
}

trait ParseOrQuit {
    fn parse_or_quit<T: FromStr>(&self, type_name: &str) -> T;
}
impl ParseOrQuit for &str {
    fn parse_or_quit<T: FromStr>(&self, type_name: &str) -> T {
        let test = self.parse::<T>();
        match test {
            Ok(value) => value,
            Err(_) => {
                error!("{} is not valid {}! Terminating", self, type_name);
                exit(0);
            }
        }
    }
}

pub fn transform_state(pair: &Pair<Rule>) -> Box<dyn StatePhi> {
    let inner_rules = pair.clone().into_inner().collect::<Vec<Pair<Rule>>>();
    match pair.as_rule() {
        Rule::Phi_and => {
            let left_rule = transform_state(inner_rules.first().unwrap());
            let right_rule = transform_state(inner_rules.get(1).unwrap());
            Box::new(AndPhi {
                left_phi: left_rule,
                right_phi: right_rule,
            })
        }
        Rule::Phi_not => {
            let inner_phi = transform_state(inner_rules.first().unwrap());
            Box::new(NotPhi { phi: inner_phi })
        }
        Rule::ap => Box::new(AP {
            value: pair.as_str().into(),
        }),
        Rule::r#true => Box::new(True {}),
        Rule::prob => {
            let inner_phi = transform_path(inner_rules.first().unwrap());
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

fn transform_path(pair: &Pair<Rule>) -> Box<dyn PathPhi> {
    let inner_rules = pair.clone().into_inner().collect::<Vec<Pair<Rule>>>();
    match pair.as_rule() {
        Rule::phi_next => {
            let inner_phi = transform_state(inner_rules.first().unwrap());
            Box::new(Next { phi: inner_phi })
        }
        Rule::phi_until => {
            let left_phi = transform_state(inner_rules.first().unwrap());
            let right_phi = transform_state(inner_rules.get(1).unwrap());
            Box::new(Until {
                prev: left_phi,
                until: right_phi,
            })
        }
        _ => panic!(),
    }
}

impl StatePhi for True {
    fn fmt(&self) -> String {
        "true".into()
    }

    fn evaluate<'a>(&'a self, model_check_info: &'a ModelCheckInfo) -> HashSet<NodeIndex> {
        model_check_info.reach_graph.node_indices().collect()
    }
}

struct AndPhi {
    left_phi: Box<dyn StatePhi>,
    right_phi: Box<dyn StatePhi>,
}

impl StatePhi for AndPhi {
    fn fmt(&self) -> String {
        format!("({}) ∧ ({})", self.left_phi, self.right_phi)
    }

    fn evaluate<'a>(&'a self, model_check_info: &'a ModelCheckInfo) -> HashSet<NodeIndex> {
        let left_markings = self.left_phi.evaluate(model_check_info);
        let right_markings = self.right_phi.evaluate(model_check_info);
        left_markings
            .intersection(&right_markings)
            .copied()
            .collect()
    }
}

impl StatePhi for NotPhi {
    fn fmt(&self) -> String {
        format!("¬ ({})", self.phi)
    }

    fn evaluate<'a>(&'a self, model_check_info: &'a ModelCheckInfo) -> HashSet<NodeIndex> {
        let phi_nodes = self.phi.evaluate(model_check_info);
        let all_nodes: HashSet<NodeIndex> = model_check_info.reach_graph.node_indices().collect();
        all_nodes.difference(&phi_nodes).copied().collect()
    }
}

impl StatePhi for Prob {
    fn fmt(&self) -> String {
        format!("P(({}), {} {})", self.phi, self.comp, self.probability)
    }

    fn evaluate<'a>(&'a self, model_check_info: &'a ModelCheckInfo) -> HashSet<NodeIndex> {
        self.phi
            .evaluate(model_check_info, &self.comp, self.probability)
    }
}

impl StatePhi for AP {
    fn fmt(&self) -> String {
        self.value.clone()
    }

    fn evaluate<'a>(&'a self, model_check_info: &'a ModelCheckInfo) -> HashSet<NodeIndex> {
        model_check_info.ap_map[&self.value].clone()
    }
}

impl PathPhi for Next {
    fn fmt(&self) -> String {
        format!("◯ ({})", self.phi)
    }

    fn evaluate<'a>(
        &'a self,
        model_check_info: &'a ModelCheckInfo,
        comp: &Comp,
        prob_bound: f64,
    ) -> HashSet<NodeIndex> {
        let graph = &model_check_info.reach_graph;
        let phi_marking_indices = self.phi.evaluate(model_check_info);
        let pre_marking_indices: HashSet<NodeIndex> = phi_marking_indices
            .iter()
            .flat_map(|index| {
                graph
                    .neighbors_directed(*index, Incoming)
                    .collect::<Vec<NodeIndex>>()
            })
            .collect();
        let mut chosen_indices: HashSet<NodeIndex> = HashSet::new();
        for pre_node_index in pre_marking_indices {
            let mut sum: f64 = 0.0;
            for phi_marking_index in &phi_marking_indices {
                let edge_index = graph.find_edge(pre_node_index, *phi_marking_index);
                if edge_index.is_none() {
                    continue;
                }
                sum += graph.edge_weight(edge_index.unwrap()).unwrap();
            }
            if comp.evaluate(sum, prob_bound) {
                chosen_indices.insert(pre_node_index);
            }
        }
        chosen_indices
    }
}

#[allow(unused_variables)]
impl Until {
    fn w_op(
        &self,
        left_tsi: &HashSet<NodeIndex>,
        right_tsi: &HashSet<NodeIndex>,
        all_indices: &HashSet<NodeIndex>,
        graph: &DiGraph<Marking, f64>,
    ) -> HashSet<NodeIndex> {
        let mut new = all_indices.clone();
        loop {
            let tmp1 = all_indices.difference(&new);
            let tmp2: HashSet<NodeIndex> = tmp1
                .flat_map(|i| graph.neighbors_directed(*i, Incoming))
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

    fn a_op(
        &self,
        left_tsi: &HashSet<NodeIndex>,
        right_tsi: &HashSet<NodeIndex>,
        all: &HashSet<NodeIndex>,
        graph: &DiGraph<Marking, f64>,
    ) -> HashSet<NodeIndex> {
        let mut new = right_tsi.clone();
        let only_pre: HashSet<NodeIndex> = graph
            .node_indices()
            .flat_map(|i| graph.neighbors_directed(i, Incoming))
            .collect();
        loop {
            let tmp1 = all.difference(&new);
            let tmp2: HashSet<NodeIndex> = tmp1
                .flat_map(|i| graph.neighbors_directed(*i, Incoming))
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
        model_check_info: &ModelCheckInfo,
        comp: &Comp,
        prob_bound: f64,
    ) -> HashSet<NodeIndex> {
        let all: HashSet<_> = model_check_info.reach_graph.node_indices().collect();

        let geq_zero = *comp == Comp::Geq && prob_bound == 0.0;
        let leq_one = *comp == Comp::Leq && prob_bound == 1.0;
        if geq_zero || leq_one {
            return all;
        }

        let left_phi = self.prev.evaluate(model_check_info);
        let right_phi = self.until.evaluate(model_check_info);

        // not E(phi_1 U phi_2) = A(not phi_2 W (not phi_1 and not phi_2))
        let not_left_phi: HashSet<NodeIndex> = all.difference(&left_phi).copied().collect();
        let not_right_phi: HashSet<NodeIndex> = all.difference(&right_phi).copied().collect();
        let not_left_and_not_right: HashSet<_> =
            not_left_phi.intersection(&not_right_phi).copied().collect();

        let left_tsi = not_right_phi;
        let right_tsi = not_left_and_not_right;

        let mut prob_map: HashMap<NodeIndex, f64> = HashMap::new();
        let s_0 = self.w_op(&left_tsi, &right_tsi, &all, &model_check_info.reach_graph);
        for index in &s_0 {
            prob_map.insert(*index, 0.0);
        }
        let s_1 = self.a_op(&left_phi, &right_phi, &all, &model_check_info.reach_graph);
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
        let graph = &model_check_info.reach_graph;
        loop {
            let mut max_error: f64 = 0.0;
            for node_index in &s_q {
                let edges: Edges<_, _> = graph.edges_directed(*node_index, Outgoing);
                let prob_to_sq: f64 = edges
                    .clone()
                    .filter(|i| s_q.contains(&i.target()))
                    .map(|i| i.weight() * prob_map.get(&i.target()).unwrap())
                    .sum();
                let prob_to_s1: f64 = edges
                    .filter(|i| s_1.contains(&i.target()))
                    .map(|i| i.weight())
                    .sum();
                let prob = prob_to_sq + prob_to_s1;
                max_error = max_error.max(prob_map[node_index] - prob);
                prob_map.insert(*node_index, prob);
            }
            if max_error < model_check_info.max_error {
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
