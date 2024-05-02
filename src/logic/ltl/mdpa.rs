use super::{safra::DRA, PhiOp, AP};
use crate::{
    input_graph::{Node, MDP},
    logic::ltl::common::Alphabet,
    mcsp::PctlInfo,
    utils::common::reverse_map,
};
use petgraph::{algo::kosaraju_scc, dot, stable_graph::NodeIndex, visit::EdgeRef, Direction};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

pub fn cross_mdp(dra: DRA, pctl_info: &PctlInfo) -> (MDP<(NodeIndex, String)>, HashSet<NodeIndex>) {
    let reversed_ap_map = reverse_map(&pctl_info.ap_map);
    let reversed_ap_map = reversed_ap_map
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                Alphabet(
                    v.into_iter()
                        .map(|ap| PhiOp::AP(AP { value: ap.into() }))
                        .collect::<BTreeSet<_>>(),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let mdp_graph = &pctl_info.reach_graph;
    let mut cross_graph: MDP<(NodeIndex, String)> = MDP::new();
    let mut pop_queue = VecDeque::new();
    pop_queue.push_back((pctl_info.initial_marking, dra.initial.clone()));
    while let Some((mdp_node, dra_state)) = pop_queue.pop_front() {
        let new_node_index = find_or_create_node(&mut cross_graph, mdp_node, &dra_state);
        let opt_props = reversed_ap_map.get(&mdp_node);

        // For all edges between State --> Action
        for edge in mdp_graph.edges(mdp_node) {
            let action = edge.target();
            if let Node::Action(action_weight) = mdp_graph.node_weight(action).unwrap() {
                let new_action_node_index =
                    cross_graph.add_node(Node::Action(action_weight.clone()));
                cross_graph.add_edge(new_node_index, new_action_node_index, *edge.weight());

                // For all edges between Action --> State
                for edge in mdp_graph.edges(action) {
                    let target_state = edge.target();
                    let target_dra_state = prop_to_state(&dra_state, opt_props, &dra);
                    let new_target_node =
                        find_or_create_node(&mut cross_graph, target_state, &target_dra_state);
                    if !(pop_queue.contains(&(target_state, target_dra_state.clone()))
                        || has_edges(&(target_state, target_dra_state.clone()), &cross_graph))
                    {
                        pop_queue.push_back((target_state, target_dra_state));
                    }
                    cross_graph.add_edge(new_action_node_index, new_target_node, *edge.weight());
                }
            } else {
                unreachable!();
            }
        }
    }

    let aec = aec(&dra.acc, &cross_graph);
    (cross_graph, aec)
}

fn aec(
    acc: &[(HashSet<String>, HashSet<String>)],
    cross_graph: &MDP<(NodeIndex, String)>,
) -> HashSet<NodeIndex> {
    // let scc = kosaraju_scc(&cross_graph);
    let mut aec: HashSet<NodeIndex> = HashSet::new();
    for (l, k) in acc {
        let mut graph = cross_graph.clone();
        for l_state in l {
            graph.retain_nodes(|g, ni| match &g[ni] {
                Node::State(s) => s.1 != *l_state,
                _ => true,
            });
            iterative_remove(&mut graph);
            let scc = kosaraju_scc(&graph);
            scc.iter()
                .filter(|c| {
                    c.len() > 1
                        && k.iter().any(|ke| {
                            c.iter().any(|ce| match &graph[*ce] {
                                Node::State(s) => s.1 == *ke,
                                Node::Action(_) => false,
                            })
                        })
                })
                .flatten()
                .for_each(|cs| {
                    aec.insert(*cs);
                });
        }
    }
    aec
}

fn iterative_remove(graph: &mut MDP<(NodeIndex, String)>) {
    loop {
        let mut remove_list: Vec<NodeIndex> = Vec::with_capacity(graph.node_count());
        graph
            .node_indices()
            .filter(|ni| match &graph[*ni] {
                Node::State(_) => false,
                Node::Action(_) => {
                    let incoming_edges_count =
                        graph.edges_directed(*ni, Direction::Incoming).count();
                    let outgoing_edges = graph.edges_directed(*ni, Direction::Outgoing);
                    let sum_weights = outgoing_edges
                        .clone()
                        .map(|e| e.weight().clone())
                        .sum::<f64>()
                        .clone()
                        != 1.0;
                    return incoming_edges_count == 0
                        || outgoing_edges.clone().count() == 0
                        || sum_weights;
                }
            })
            .for_each(|ni| remove_list.push(ni));
        if remove_list.is_empty() {
            break;
        } else {
            for ni in &remove_list {
                let _ = graph.remove_node(*ni);
            }
        }

        // Remove all states with
        graph
            .node_indices()
            .filter(|ni| match graph[*ni] {
                Node::Action(_) => false,
                Node::State(_) => graph.edges_directed(*ni, Direction::Outgoing).count() == 0,
            })
            .for_each(|ni| remove_list.push(ni));
        if remove_list.is_empty() {
            break;
        } else {
            for ni in remove_list {
                let _ = graph.remove_node(ni);
            }
        }
    }
}

fn prop_to_state(src_state: &String, opt_alphabet: Option<&Alphabet>, dra: &DRA) -> String {
    let full_alph = Alphabet::full();
    dra.delta(
        src_state,
        match opt_alphabet {
            Some(alph) => alph,
            None => &full_alph,
        },
    )
}

fn find_or_create_node(
    cross_graph: &mut MDP<(NodeIndex, String)>,
    mdp_node: NodeIndex,
    dra_state: &String,
) -> NodeIndex {
    match cross_graph
        .node_indices()
        .find(|i| cross_graph[*i] == Node::State((mdp_node, dra_state.into())))
    {
        Some(index) => index,
        None => cross_graph.add_node(Node::State((mdp_node, dra_state.into()))),
    }
}

fn has_edges(node: &(NodeIndex, String), graph: &MDP<(NodeIndex, String)>) -> bool {
    graph
        .edges(
            graph
                .node_indices()
                .find(|&i| graph[i] == Node::State(node.clone()))
                .unwrap(),
        )
        .count()
        > 0
}
