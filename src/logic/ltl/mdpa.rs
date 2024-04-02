use super::safra::DRA;
use crate::{
    input_graph::{Node, MDP},
    mcsp::PctlInfo,
    utils::common::reverse_map,
};
use petgraph::{algo::kosaraju_scc, graph::NodeIndex, visit::EdgeRef};
use std::collections::{HashSet, VecDeque};

pub fn cross_mdp(dra: DRA, pctl_info: &PctlInfo) -> (MDP<(NodeIndex, String)>, HashSet<NodeIndex>) {
    let reversed_ap_map = reverse_map(&pctl_info.ap_map);
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
                    let target_dra_states = prop_to_state(&dra_state, opt_props, &dra);
                    for target_dra_state in target_dra_states {
                        let new_target_node =
                            find_or_create_node(&mut cross_graph, target_state, &target_dra_state);
                        if !(pop_queue.contains(&(target_state, target_dra_state.clone()))
                            || has_edges(&(target_state, target_dra_state.clone()), &cross_graph))
                        {
                            pop_queue.push_back((target_state, target_dra_state));
                        }
                        cross_graph.add_edge(
                            new_action_node_index,
                            new_target_node,
                            *edge.weight(),
                        );
                    }
                }
            } else {
                unreachable!();
            }
        }
    }

    let scc = kosaraju_scc(&cross_graph);
    let aec = aec(scc, &dra.acc, &cross_graph);
    return (cross_graph, aec);
}

fn aec(
    scc: Vec<Vec<NodeIndex>>,
    acc: &[(HashSet<String>, HashSet<String>)],
    cross_graph: &petgraph::prelude::Graph<Node<(NodeIndex, String)>, f64>,
) -> HashSet<NodeIndex> {
    let mut aec: HashSet<NodeIndex> = HashSet::new();
    for component in scc {
        // Our graph nodes are divided into state nodes and action nodes. So we need to consider
        // components containing the corresponding state and action nodes and ignore the rest
        if component.len() < 2 {
            continue;
        }

        for acc_pair in acc {
            let is_not_in_l = component
                .iter()
                .filter_map(|ni| match cross_graph[*ni].clone() {
                    Node::State(state) => Some(state),
                    Node::Action(_) => None,
                })
                .all(|(_, q)| !acc_pair.0.contains(&q));
            let is_in_k = component
                .iter()
                .filter_map(|ni| match cross_graph[*ni].clone() {
                    Node::State(state) => Some(state),
                    Node::Action(_) => None,
                })
                .any(|(_, q)| acc_pair.1.contains(&q));

            if is_not_in_l && is_in_k {
                aec.extend(component.iter().filter(|&ni| cross_graph[*ni].is_state()));
            }
        }
    }
    aec
}

fn prop_to_state(
    src_state: &String,
    opt_props: Option<&HashSet<&String>>,
    dra: &DRA,
) -> Vec<String> {
    if let Some(props) = opt_props {
        return props
            .into_iter()
            .map(|prop| dra.delta(&src_state, Some(&prop)))
            .collect();
    } else {
        return vec![dra.delta(&src_state, None)];
    }
}

fn find_or_create_node(
    cross_graph: &mut petgraph::prelude::Graph<Node<(NodeIndex, String)>, f64>,
    mdp_node: NodeIndex,
    dra_state: &String,
) -> NodeIndex {
    let new_node_index = match cross_graph
        .node_indices()
        .find(|i| cross_graph[*i] == Node::State((mdp_node, dra_state.into())))
    {
        Some(index) => index,
        None => cross_graph.add_node(Node::State((mdp_node, dra_state.into()))),
    };
    new_node_index
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
