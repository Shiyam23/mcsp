use petgraph::{algo::dijkstra, graph::NodeIndex, prelude::DiGraph};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    process::exit,
};
use log::warn;
use crate::input_graph;
use super::{ApMap, GenericApMap, GenericMDP, InputGraph, MDP, Node::{Action, State}, Node};

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

pub struct Transition {
    pub transition_id: usize,
    pub name: String,
    pub pre: Vec<usize>,
    pub succ: Vec<usize>,
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

pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub initial_marking: Marking,
    pub ap_map: ApMap<Marking>
}

impl PetriNet {
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
                    return input_graph::State::le(m, marking) && *m != *marking
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
            .filter(|t| t.pre.iter().all(|state_id| marking[*state_id] > 0))
            .collect()
    }

    fn succ_marking(marking: &[usize], transition: &Transition) -> Vec<usize> {
        let mut succ_marking = marking.to_owned();
        for state_id in &transition.pre {
            succ_marking[*state_id] -= 1;
        }
        for state_id in &transition.succ {
            succ_marking[*state_id] += 1;
        }
        succ_marking
    }

    pub fn to_mdp(&self) -> MDP<Marking> {
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

            // Add pseudo action
            let pseudo_action: Node<_> = Action("".into());
            let action_index = reach_graph.add_node(pseudo_action);
            reach_graph.add_edge(pre_index, action_index, 1.0);
            // Add transitions
            let active_transitions = PetriNet::get_active_transitions(&marking, &self.transitions);
            let sum_fire_rates: f64 = active_transitions.iter().map(|t| t.fire_rate).sum();
            for active_transition in active_transitions {
                let succ_marking = PetriNet::succ_marking(&marking, active_transition);
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
                PetriNet::check_infinite_graph(&reach_graph, &marking, &pre_index);
                let probability = active_transition.fire_rate / sum_fire_rates;
                reach_graph.add_edge(action_index, succ_index, probability);
            }
        }
        reach_graph
    }
}

impl InputGraph for PetriNet {
    type S = Marking;
    fn validate_graph(&mut self) {
        let graph = self.to_mdp();
        let graph_markings: Vec<&Marking> = graph
            .node_weights()
            .filter_map(|n| match n {
                State(s) => Some(s),
                Action(_) => None
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
        self.ap_map.retain(|k,v| {
            if v.is_empty() {
                warn!(
                    "\"{}\" is empty! Removing it from the list of all AP's", k
                )
            }
            !v.is_empty()
        });
    }

    fn to_mdp(&self) -> MDP<Marking> {
        self.to_mdp()
    }

    fn get_ap_map(&self) -> &ApMap<Marking> { &self.ap_map }

    fn get_init_state(&self) -> &Marking { &self.initial_marking }
}

impl GenericMDP for MDP<Marking> {}
impl GenericApMap for ApMap<Marking> {}
impl input_graph::State for Marking {
    fn le(&self, other: &Self) -> bool{
        for index in 0..self.len() {
            if self[index] > other[index] {
                return false
            }
        }
        true
    }
}