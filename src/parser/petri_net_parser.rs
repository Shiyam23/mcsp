use std::process::exit;
use std::str::FromStr;
use log::error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use crate::input_graph::{ApMap, ParseImpl};
use crate::input_graph::pnet::{Marking, PetriNet, Place, Transition};
use crate::utils::common::ParseOrQuit;

const GRAPH_ID: &str = "G";
const INITIAL_MARKINGS_ID: &str = "M";
const LAMBDAS_ID: &str = "L";
const AP_MAP_ID: &str = "AP_MAP";

#[derive(Parser)]
#[grammar = "parser/petri_net.pest"]
pub struct InputParser;

pub struct PetriNetParser;

impl ParseImpl<PetriNet> for PetriNetParser {
    fn parse(content: &str) -> Box<PetriNet> {
        let pairs = InputParser::parse(Rule::Main, content).unwrap();
        let initial_marking = parse_list::<usize>(&pairs.find_first_tagged(INITIAL_MARKINGS_ID).unwrap().into_inner());
        let lambdas = parse_list::<f64>(&pairs.find_first_tagged(LAMBDAS_ID).unwrap().into_inner());
        let mut places_raw: Vec<&str> = Vec::new();
        let mut transitions: Vec<Transition> = Vec::new();
        let graph_rule = pairs.find_first_tagged(GRAPH_ID).unwrap();

        // Check whether there are the same no. of fire ratings as transitions
        let t_assigns = graph_rule.into_inner();
        if t_assigns.len() != lambdas.len() {
            error!("{} fire rates were detected but there are {} transitions! Aborting...", lambdas.len(), t_assigns.len());
            exit(0);
        }

        // Get all place names
        for (t_index, t_assign) in t_assigns.enumerate(){
            let both_place_rules = [Rule::input_p, Rule::output_p]
                .into_iter()
                .map(|rule|  t_assign.clone().into_inner().find(|r| r.as_rule() == rule))
                .map(|rule_result| { rule_result.map(|rule| rule.into_inner())})
                .collect::<Vec<_>>();

            let mut input_p_indices : Vec<usize> = Vec::new();
            let mut output_p_indices : Vec<usize> = Vec::new();
            if let Some(input_rules) = both_place_rules[0].clone() {
                for place_rule in input_rules.clone() {
                    if !places_raw.contains(&place_rule.as_str()) {
                        places_raw.push(place_rule.as_str());
                    }
                    input_p_indices.push(places_raw.iter().position(|s| *s == place_rule.as_str()).unwrap());
                }
            }
            if let Some(output_rules) = both_place_rules[1].clone() {
                for place_rule in output_rules.clone() {
                    if !places_raw.contains(&place_rule.as_str()) {
                        places_raw.push(place_rule.as_str());
                    }
                    output_p_indices.push(places_raw.iter().position(|s| *s == place_rule.as_str()).unwrap());
                }
            }
            transitions.push(
                Transition {
                    transition_id: t_index,
                    name: "".to_string(),
                    pre: input_p_indices,
                    succ: output_p_indices,
                    fire_rate: lambdas[t_index],
                }
            );
        }

        // If markings and detected places in transition assignments not equal, exit with error msg
        if places_raw.len() != initial_marking.len() {
            error!("{} places were detected but initial marking has {} places", places_raw.len(), initial_marking.len());
            error!("Detected places: {:?}", places_raw);
            exit(0);
        }

        // Transform place names to actual places
        let places = places_raw
            .into_iter()
            .enumerate()
            .map(|(index, name)| Place {
                state_id: index,
                name: name.to_string(),
                token: initial_marking[index],
            })
            .collect::<Vec<Place>>();


        let ap_map_pairs = pairs.find_first_tagged(AP_MAP_ID).unwrap();
        let ap_map = transform_ap_map(ap_map_pairs);
        let petri_net = PetriNet {
            places,
            transitions,
            ap_map,
            initial_marking
        };
        Box::new(petri_net)
    }
}

fn parse_list<T>(list: &Pairs<Rule>) -> Vec<T> where T: FromStr {
    let mut tmp_vec: Vec<T> = Vec::new();
    for rule in list.clone() {
        let input_string = rule.as_str();
        match input_string.parse::<T>() {
            Ok(value) => tmp_vec.push(value),
            Err(_) =>  panic!("{} is not a valid! Terminating...", input_string)
            }
        }
    tmp_vec
}

pub fn transform_ap_map(pair: Pair<Rule>) -> ApMap<Marking> {
    assert_eq!(pair.as_rule(), Rule::AP);
    let mut ap_map = ApMap::new();
    for ap_assign in pair.clone().into_inner() {
        let elements: Vec<Pair<Rule>> = ap_assign.into_inner().collect();
        match (elements.first(), elements.get(1)) {
            (Some(ap), Some(markings_rule)) => {
                let markings = markings_rule
                    .clone()
                    .into_inner()
                    .map(|marking| {
                        marking
                            .into_inner()
                            .map(|int_rule| int_rule.as_str().parse_or_quit("integer"))
                            .collect()
                    })
                    .collect();
                ap_map.insert(ap.as_str().into(), markings);
            }
            (_, _) => panic!(),
        }
    }
    ap_map
}