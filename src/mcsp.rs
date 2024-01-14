use std::{
    collections::{HashMap, HashSet},
};
use std::fmt::Debug;
use std::marker::PhantomData;
use log::info;
use petgraph::graph::{NodeIndex};
use crate::{Args};
use crate::input_graph::{ApMap, InputGraph, MDP, Node, ParseImpl};
use crate::input_graph::Node::{Action, State};
use crate::logic::{Formula, parse_formula};
use crate::utils::file::read_file;

pub struct ModelCheckInfo<'a, T> {
    pub reach_graph: MDP<T>,
    pub ap_map: &'a ApMap<T>,
    pub formula: Box<dyn Formula>,
    pub max_error: f64
}

pub struct PctlInfo {
    pub reach_graph: MDP<NodeIndex>,
    pub ap_map: HashMap<String, HashSet<NodeIndex>>,
    pub max_error: f64
}

pub struct ModelCheck<T: InputGraph, P: ParseImpl<T>> {
    p1: PhantomData<T>,
    p2: PhantomData<P>,
}

impl<T, P> ModelCheck<T, P> where T: InputGraph, P: ParseImpl<T>{
    //noinspection RsTraitObligations
    pub fn start(args: Args) {
        info!("Parsing input file");
        let content = read_file(&args.input_file);
        let input_graph: Box<T> = P::parse(&content);
        let formula = parse_formula(args.logic_type, &content);
        input_graph.validate_graph();
        let mc: ModelCheckInfo<T::S> = ModelCheckInfo{
            reach_graph: input_graph.to_mdp(),
            ap_map: input_graph.get_ap_map(),
            formula,
            max_error: args.max_error,
        };
        Self::evaluate_pctl(mc);
    }

    pub fn evaluate_pctl<K>(mc_info: ModelCheckInfo<K>) where K: Debug + PartialEq + Clone {
        let normalized_mdp = mc_info.reach_graph.map(
            |ni,node| match node {
                State(_) => {State(ni)}
                Action(a) => {Action(a.into())}
            },
            |_,e| *e
        );

        let normalized_ap_map = Self::normalize_ap_map(&mc_info.reach_graph, mc_info.ap_map);
        let pctl_info: PctlInfo = PctlInfo{
            reach_graph: normalized_mdp,
            ap_map: normalized_ap_map,
            max_error: mc_info.max_error
        };

        let markings = mc_info.formula.evaluate(&pctl_info);

        // Print all markings satisfying the pctl statement
        info!("The following markings satisfy the given pctl statement:");
        info!("{:?}",
            markings
                .iter()
                .map(|index| mc_info.reach_graph[*index].clone())
                .collect::<Vec<Node<K>>>()
        );
    }

    fn normalize_ap_map<K>(
        graph: &MDP<K>,
        src_map: &HashMap<String, HashSet<K>>,
    ) -> HashMap<String, HashSet<NodeIndex>> where K: PartialEq{

        let mut normalized_ap_map: HashMap<String, HashSet<NodeIndex>> = HashMap::new();
        for (ap, markings) in src_map {
            normalized_ap_map.insert(ap.into(), HashSet::new());
            for marking in markings {
                let node_index = graph.node_indices().find(|&node_index| match &graph[node_index] {
                    State(m) => m == marking,
                    Action(_) => false
                }).unwrap();
                normalized_ap_map.get_mut(ap).unwrap().insert(node_index);
            }
        }
        normalized_ap_map
    }


}
