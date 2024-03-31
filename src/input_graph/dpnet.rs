use super::{
    ApMap, InputGraph, Node,
    Node::{Action, State},
    MDP,
};
use crate::input_graph;
use crate::utils::common::powerset;
use log::warn;
use petgraph::{algo::dijkstra, graph::NodeIndex, prelude::DiGraph};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    process::exit,
};

pub type Marking = Vec<usize>;

#[derive(Debug)]
pub struct Place {
    pub state_id: usize,
    pub name: String,
    pub token: usize,
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {}, Token: {}", self.name, self.token)
    }
}

#[derive(PartialEq)]
pub struct Transition {
    pub transition_id: usize,
    pub name: String,
    pub pre: Vec<(usize, usize)>,
    pub succ: Vec<(usize, usize)>,
    pub fire_rate: f64,
}

impl Clone for Transition {
    fn clone(&self) -> Self {
        Self {
            transition_id: self.transition_id,
            name: self.name.clone(),
            pre: self.pre.clone(),
            succ: self.succ.clone(),
            fire_rate: self.fire_rate,
        }
    }
}

impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.fire_rate)
    }
}

pub struct DPetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub c_transitions: Vec<Transition>,
    pub initial_marking: Marking,
    pub ap_map: ApMap<Marking>,
}

impl DPetriNet {
    fn check_infinite_graph<'a>(
        graph: &'a MDP<Marking>,
        marking: &Marking,
        marking_index: &NodeIndex,
    ) -> Option<&'a Marking> {
        // Get all smaller markings
        let node_indices: Vec<_> = graph
            .node_indices()
            .filter(|&m| {
                if let State(m) = &graph[m] {
                    return input_graph::State::le(m, marking) && *m != *marking;
                }
                false
            })
            .collect();
        if node_indices.is_empty() {
            return None;
        };

        //Check whether these smaller markings are connected to this one
        for node_index in node_indices {
            let path_res = dijkstra(graph, node_index, Some(*marking_index), |_| 1);
            if path_res.get(marking_index).is_some() {
                println!(
                    "There is a path from {:?} to {:?}! That is why the graph is infinite",
                    &graph[node_index], marking
                );
                exit(0);
            }
        }
        None
    }

    fn get_active_transitions<'a>(
        marking: &Marking,
        transitions: &'a [Transition],
    ) -> Vec<&'a Transition> {
        transitions
            .iter()
            .filter(|t| {
                t.pre
                    .iter()
                    .all(|(state_id, tokens)| marking[*state_id] >= *tokens)
            })
            .collect()
    }

    fn succ_marking(marking: &[usize], transition: &Transition) -> Vec<usize> {
        let mut succ_marking = marking.to_owned();
        for (state_id, tokens) in &transition.pre {
            succ_marking[*state_id] -= tokens;
        }
        for (state_id, tokens) in &transition.succ {
            succ_marking[*state_id] += tokens;
        }
        succ_marking
    }

    pub fn to_mdp(&self, precision: i32) -> (MDP<Marking>, Marking) {
        let mut reach_graph: MDP<Marking> = DiGraph::new();
        let states: &Vec<Place> = &self.places;
        let initial_marking: Marking = states.iter().map(|s| s.token).collect();
        let mut upcoming_markings = VecDeque::<Marking>::new();
        upcoming_markings.push_back(initial_marking.clone());
        reach_graph.add_node(State(initial_marking));
        while let Some(marking) = upcoming_markings.pop_front() {
            let pre_index = reach_graph
                .node_indices()
                .find(|&n| reach_graph[n] == State(marking.clone()))
                .unwrap();
            let enabled_transitions =
                DPetriNet::get_active_transitions(&marking, &self.transitions);
            let deactivated_transitions: Vec<&Transition> = self
                .c_transitions
                .iter()
                .filter(|t| enabled_transitions.contains(t))
                .collect();
            for deactivated_transition_set in powerset(&deactivated_transitions) {
                // Add pseudo action
                let pseudo_action: Node<_> = Action(fmt(&deactivated_transition_set));
                let action_index = reach_graph.add_node(pseudo_action);
                reach_graph.add_edge(pre_index, action_index, 1.0);
                let new_activated_transitions: Vec<&&Transition> = enabled_transitions
                    .iter()
                    .filter(|t| !deactivated_transition_set.contains(t))
                    .collect();

                // Add transitions
                let sum_fire_rates: f64 =
                    new_activated_transitions.iter().map(|t| t.fire_rate).sum();

                // If there are no activated transitions, then add one edge back to marking with probabilty 1.0
                if new_activated_transitions.is_empty() {
                    reach_graph.add_edge(action_index, pre_index, 1.0);
                }

                // Otherwise iterate through all activated transitions
                for activated_transition in new_activated_transitions {
                    let succ_marking = DPetriNet::succ_marking(&marking, activated_transition);
                    let succ_index;
                    if let Some(index) = reach_graph
                        .node_indices()
                        .find(|&n| reach_graph[n] == State(succ_marking.clone()))
                    {
                        succ_index = index;
                    } else {
                        succ_index = reach_graph.add_node(State(succ_marking.clone()));
                        upcoming_markings.push_back(succ_marking);
                    }
                    DPetriNet::check_infinite_graph(&reach_graph, &marking, &pre_index);
                    let mut probability = activated_transition.fire_rate / sum_fire_rates;
                    probability =
                        (probability * 10.0_f64.powi(precision)).round() / 10.0_f64.powi(precision);
                    reach_graph.add_edge(action_index, succ_index, probability);
                }
            }
        }
        (reach_graph, initial_marking)
    }
}

impl InputGraph for DPetriNet {
    type S = Marking;
    fn validate_graph(&mut self, graph: &MDP<Marking>) {
        let graph_markings: Vec<&Marking> = graph
            .node_weights()
            .filter_map(|n| match n {
                State(s) => Some(s),
                Action(_) => None,
            })
            .collect();
        // Check whether the assigned markings are reached (is a node in the reachability graph)
        self.ap_map
            .iter_mut()
            .for_each(|(ap, v)| v.retain(|m| {
                let retain: bool = graph_markings.contains(&m);
                if !retain {
                    warn!(
                        "{:?} was assigned to \"{}\" but is never reached! Removing from \"{}\" ...",
                        m, ap, ap
                    );
                }
                retain
            }));
        self.ap_map.retain(|k, v| {
            if v.is_empty() {
                warn!("\"{}\" is empty! Removing it from the list of all AP's", k)
            }
            !v.is_empty()
        });
    }

    fn to_mdp(&self, precision: i32) -> (MDP<Marking>, Marking) {
        self.to_mdp(precision)
    }

    fn get_ap_map(&self) -> &ApMap<Marking> {
        &self.ap_map
    }

    fn get_init_state(&self) -> &Marking {
        &self.initial_marking
    }
}

fn fmt(list: &[&&Transition]) -> String {
    let mut list_name: String = "{".into();
    if let Some(item) = list.first() {
        list_name.push_str(&item.name);
    }
    for item in list.iter().skip(1) {
        list_name.push_str(", ");
        list_name.push_str(&item.name);
    }
    list_name.push('}');
    list_name.to_owned()
}
