#[derive(Debug)]
pub struct State {
    pub state_id: usize,
    pub name: String,
    pub token: usize,
}

impl Display for State {
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
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
}

use crate::utils::pnet_parser::PetriNetInfo;
use petgraph::{algo::dijkstra, graph::NodeIndex, prelude::DiGraph};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    process::exit,
};

pub type Marking = Vec<usize>;

impl PetriNet {
    pub fn from_info(p_info: &PetriNetInfo) -> PetriNet {
        let states: Vec<State> = p_info
            .places
            .iter()
            .enumerate()
            .map(|(index, place)| State {
                state_id: index,
                name: place.clone(),
                token: *p_info.initial_marking.get(index).unwrap(),
            })
            .collect();
        let pre_states_from_pairs = |transition: &String| {
            p_info
                .input_arcs
                .iter()
                .filter(|(_, t)| t == transition)
                .map(|(s, _)| {
                    states
                        .iter()
                        .find(|state| state.name == *s)
                        .unwrap()
                        .state_id
                })
                .collect::<Vec<usize>>()
        };
        let succ_states_from_pairs = |transition: &String| {
            p_info
                .output_arcs
                .iter()
                .filter(|(t, _)| t == transition)
                .map(|(_, s)| {
                    states
                        .iter()
                        .find(|state| state.name == *s)
                        .unwrap()
                        .state_id
                })
                .collect::<Vec<usize>>()
        };
        let transitions: Vec<Transition> = p_info
            .transitions
            .iter()
            .enumerate()
            .map(|(index, transition)| Transition {
                transition_id: index,
                name: transition.clone(),
                pre: pre_states_from_pairs(transition),
                succ: succ_states_from_pairs(transition),
                fire_rate: *p_info.lambdas.get(index).unwrap(),
            })
            .collect();
        PetriNet {
            states,
            transitions,
        }
    }

    pub fn get_reach_graph(&self) -> DiGraph<Marking, f64> {
        let mut reach_graph: DiGraph<Marking, f64> = DiGraph::new();
        let states: &Vec<State> = &self.states;
        let initial_marking: Marking = states.iter().map(|s| s.token).collect();
        let mut upcoming_markings = VecDeque::<Marking>::new();
        upcoming_markings.push_back(initial_marking.clone());
        reach_graph.add_node(initial_marking);
        while let Some(marking) = upcoming_markings.pop_front() {
            let pre_index = reach_graph
                .node_indices()
                .find(|&n| reach_graph[n] == marking)
                .unwrap();
            let active_transitions = PetriNet::get_active_transitions(&marking, &self.transitions);
            let sum_fire_rates: f64 = active_transitions.iter().map(|t| t.fire_rate).sum();
            for active_transition in active_transitions {
                let succ_marking = PetriNet::succ_marking(&marking, active_transition);
                let succ_index;
                if let Some(index) = reach_graph
                    .node_indices()
                    .find(|&n| reach_graph[n] == succ_marking)
                {
                    succ_index = index;
                } else {
                    succ_index = reach_graph.add_node(succ_marking.clone());
                    upcoming_markings.push_back(succ_marking);
                }
                PetriNet::check_infinite_graph(&reach_graph, &marking, &pre_index);
                let probability = active_transition.fire_rate / sum_fire_rates;
                reach_graph.add_edge(pre_index, succ_index, probability);
            }
        }
        reach_graph
    }

    fn check_infinite_graph<'a, T>(
        graph: &'a DiGraph<Marking, T>,
        marking: &Marking,
        marking_index: &NodeIndex,
    ) -> Option<&'a Marking> {
        // Get all smaller markings
        let node_indices: Vec<_> = graph
            .node_indices()
            .filter(|&m| graph[m].le(marking) && graph[m] != *marking)
            .collect();
        if node_indices.is_empty() {
            return Option::None;
        };

        //Check whether these smaller markings are connected to this one
        for node_index in node_indices {
            let path_res = dijkstra(graph, node_index, Some(*marking_index), |_| 1);
            if path_res.get(marking_index).is_some() {
                println!(
                    "There is a path from {:?} to {:?}! Thats why the graph is infinite",
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

    fn succ_marking(marking: &Vec<usize>, transition: &Transition) -> Vec<usize> {
        let mut succ_marking = marking.clone();
        for state_id in &transition.pre {
            succ_marking[*state_id] -= 1;
        }
        for state_id in &transition.succ {
            succ_marking[*state_id] += 1;
        }
        succ_marking
    }
}
