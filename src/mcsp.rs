use std::{
    collections::{HashMap, HashSet},
    process::exit,
};

use crate::parser::mc_parser::StatePhi;
use crate::parser::parser::{parse, InputData};
use crate::utils::pnet::{Marking, PetriNet};
use log::{error, info};
use petgraph::{graph::DiGraph, graph::NodeIndex};

pub struct ModelCheckInfo {
    pub reach_graph: DiGraph<Marking, f64>,
    pub initial_marking: Marking,
    pub ap_map: HashMap<String, HashSet<NodeIndex>>,
    pub pctl: Box<dyn StatePhi>,
    pub max_error: f64
}

impl ModelCheckInfo {
    pub fn parse(input_file: &str, max_error: f64) -> ModelCheckInfo {
        info!("Starting MCSP...");

        // Parsing petri net
        info!("Parsing petri net...");
        let mut input_data: InputData = parse(input_file);
        let initial_marking: Marking = input_data.petri_net.initial_marking.to_owned();
        let pnet: PetriNet = PetriNet::from_info(&input_data.petri_net);
        info!("Successfully parsed petri net!");

        // Reachability Graph
        info!("Creating reachability graph for given petri net ...");
        let reach_graph = pnet.get_reach_graph();
        //println!("{:?}", Dot::new(&reach_graph));
        info!("Successfully created reachability graph");

        let new_ap_map = Self::map_marking_to_node_indices(&reach_graph, &input_data.ap_map);

        //Validate mc_info
        Self::validate_mc_info(&mut input_data, &reach_graph);
        ModelCheckInfo {
            reach_graph,
            initial_marking,
            ap_map: new_ap_map,
            pctl: input_data.phi,
            max_error
        }
    }

    pub fn evaluate_pctl(&self) {
        let markings = self.pctl.evaluate(self);
        //println!("{:?}", markings);
        println!(
            "{:?}",
            markings
                .iter()
                .map(|index| &self.reach_graph[*index])
                .collect::<Vec<&Marking>>()
        );
    }

    fn map_marking_to_node_indices(
        graph: &DiGraph<Vec<usize>, f64>,
        src_map: &HashMap<String, Vec<Marking>>,
    ) -> HashMap<String, HashSet<NodeIndex>> {
        src_map
            .iter()
            .map(|(k, v)| {
                (
                    k.to_owned(),
                    v.iter()
                        .map(|m| graph.node_indices().find(|i| graph[*i] == *m).unwrap())
                        .collect(),
                )
            })
            .collect()
    }

    fn validate_mc_info(mc_info: &mut InputData, graph: &DiGraph<Marking, f64>) {
        let graph_markings: Vec<&Marking> = graph.node_weights().collect();

        // Check whether the assigned markings are reached (is a node in the reachability graph)
        for ap in mc_info.ap_map.keys() {
            for assigned_marking in &mc_info.ap_map[ap] {
                if !graph_markings.contains(&assigned_marking) {
                    error!(
                        "{:?} was assigned to \"{}\" but is never reached! Terminating ...",
                        assigned_marking, ap
                    );
                    exit(0);
                }
            }
        }
    }
}
