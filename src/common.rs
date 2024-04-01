use crate::input_graph::{Node, MDP};
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;

pub fn rename_map<T>(mdp: &MDP<T>) -> BTreeMap<Node<T>, NodeIndex>
where
    T: Eq + Ord + Clone,
{
    mdp.node_indices().map(|ni| (mdp[ni].clone(), ni)).collect()
}
