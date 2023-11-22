use std::{
    collections::{HashMap, HashSet},
    process::exit,
};

use crate::utils::{
    mc_parser::{self, MCInfo, StatePhi},
    pnet::{Marking, PetriNet},
    pnet_parser::{parse, PetriNetInfo},
};
use log::{error, info};
use petgraph::{graph::DiGraph, graph::NodeIndex};

#[allow(dead_code)]
pub struct ModelCheckInfo {
    pub reach_graph: DiGraph<Marking, f64>,
    pub initial_marking: Marking,
    pub ap_map: HashMap<String, HashSet<NodeIndex>>,
    pub pctl: Box<dyn StatePhi>,
}

impl ModelCheckInfo {
    pub fn parse(p_file: &str, mc_file: &str) -> ModelCheckInfo {
        info!("Starting MCSP...");

        // Parsing petri net
        info!("Parsing petri net...");
        let p_info: PetriNetInfo = parse(p_file);
        let initial_marking: Marking = p_info.initial_marking.to_owned();
        let pnet: PetriNet = PetriNet::from_info(&p_info);
        drop(p_info);
        info!("Succesfully parsed petri net!");

        // Parsing modelcheck info
        info!("Parsing Modelcheck Info ...");
        let mut mc_info = mc_parser::parse(mc_file);
        info!("Succesfully parsed Modelcheck Info");

        // Reachability Graph
        info!("Creating reachability graph for given petri net ...");
        let reach_graph = pnet.get_reach_graph();
        //println!("{:?}", dot::Dot::new(&reach_graph));
        info!("Succesfully created reachability graph");

        let new_ap_map = Self::map_marking_to_node_indices(&reach_graph, &mc_info.ap_map);

        //Validate mc_info
        Self::validate_mc_info(&mut mc_info, &reach_graph);

        ModelCheckInfo {
            reach_graph,
            initial_marking,
            ap_map: new_ap_map,
            pctl: mc_info.phi,
        }
    }

    pub fn evaluate_pctl(&self) {
        //println!("{}", self.pctl);
        let markings = self.pctl.evaluate(self);
        //println!("{:?}", markings);
        println!("{:?}", markings.iter().map(|index| &self.reach_graph[*index]).collect::<Vec<&Marking>>());
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

    fn validate_mc_info(mc_info: &mut MCInfo, graph: &DiGraph<Marking, f64>) {
        let graph_markings: Vec<&Marking> = graph.node_weights().collect();

        // Check whether all ap's in ap_map were declared before
        if let Some(ap) = mc_info.ap_map.keys().find(|ap| !mc_info.ap.contains(ap)) {
            error!(
                "\"{}\" is assigned a marking but is not defined in the set",
                ap
            );
            exit(0);
        };

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
