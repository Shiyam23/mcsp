use crate::common::rename_map;
use crate::input_graph::Node::{Action, State};
use crate::input_graph::{ApMap, InputGraph, Node, ParseImpl, MDP};
use crate::logic::{parse_formula, Formula};
use crate::utils::common::reverse_btree_map;
use crate::utils::file::read_file;
use crate::Args;
use log::info;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct ModelCheckInfo<'a, T> {
    pub initial_marking: T,
    pub reach_graph: MDP<T>,
    pub ap_map: &'a ApMap<T>,
    pub formula: Formula,
    pub max_error: f64,
}

pub struct PctlInfo {
    pub initial_marking: NodeIndex,
    pub reach_graph: MDP<NodeIndex>,
    pub ap_map: HashMap<String, HashSet<NodeIndex>>,
    pub max_error: f64,
}

pub struct ModelCheck<T: InputGraph, P: ParseImpl<T>> {
    p1: PhantomData<T>,
    p2: PhantomData<P>,
}

impl<T, P> ModelCheck<T, P>
where
    <T as InputGraph>::S: Ord,
    T: InputGraph,
    P: ParseImpl<T>,
{
    pub fn start(args: Args) {
        info!("Parsing input petri net");
        let content = read_file(&args.input_file);
        let mut input_graph: Box<T> = P::parse(&content);
        info!("Petri net parsed successfully");
        info!("Validating petri net...");
        let (reach_graph, initial_marking) = input_graph.to_mdp(args.precision_digits);
        input_graph.validate_graph(&reach_graph);
        info!("Petri net has been validated successfully");

        // Show graph if user requests
        if args.show_graph {
            info!("Dot graph as requested...");
            println!("{:?}", Dot::new(&reach_graph));
        }

        info!("Parsing formula...");
        let formula = parse_formula(args.logic_type, &content);
        info!("Formula parsed successfully");
        let mc: ModelCheckInfo<T::S> = ModelCheckInfo {
            initial_marking,
            reach_graph,
            ap_map: input_graph.get_ap_map(),
            formula,
            max_error: args.max_error,
        };
        Self::evaluate_pctl(mc);
    }

    pub fn evaluate_pctl<K>(mc_info: ModelCheckInfo<K>)
    where
        K: Debug + PartialEq + Clone + Ord,
    {
        let rename_map = rename_map(&mc_info.reach_graph);
        let normalized_mdp = mc_info.reach_graph.map(
            |_, node| match node {
                State(_) => State(*rename_map.get(node).unwrap()),
                Action(a) => Action(a.clone()),
            },
            |_, e| *e,
        );
        let initial_node = *rename_map
            .get(&Node::State(mc_info.initial_marking))
            .unwrap();
        let normalized_ap_map = mc_info
            .ap_map
            .iter()
            .map(|(ap, set)| {
                let renamed_set = set
                    .iter()
                    .map(|k| *rename_map.get(&State(k.clone())).unwrap())
                    .collect();
                (ap.into(), renamed_set)
            })
            .collect();
        let pctl_info: PctlInfo = PctlInfo {
            initial_marking: initial_node,
            reach_graph: normalized_mdp,
            ap_map: normalized_ap_map,
            max_error: mc_info.max_error,
        };

        info!("Evaluating formula...");
        mc_info
            .formula
            .evaluate(&pctl_info, reverse_btree_map(rename_map));
    }
}
