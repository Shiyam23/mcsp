use crate::input_graph::pnet::{Marking, PetriNet, Place, Transition};
use crate::input_graph::{ApMap, ParseImpl};
use crate::utils::common::ParseOrQuit;
use log::{error, warn};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::process::exit;
use std::str::FromStr;

const PLACES_ID: &str = "P";
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
        let initial_marking = parse_list::<usize>(
            &pairs
                .find_first_tagged(INITIAL_MARKINGS_ID)
                .unwrap()
                .into_inner(),
        );
        let lambdas = parse_list::<f64>(&pairs.find_first_tagged(LAMBDAS_ID).unwrap().into_inner());
        let all_places = pairs
            .find_first_tagged(PLACES_ID)
            .unwrap()
            .into_inner()
            .map(|pair| pair.as_str())
            .collect::<Vec<_>>();
        let mut transitions: Vec<Transition> = Vec::new();
        let graph_rule = pairs.find_first_tagged(GRAPH_ID).unwrap();

        // Check whether there are the same no. of fire ratings as transitions
        let t_assigns = graph_rule.into_inner();
        if t_assigns.len() != lambdas.len() {
            error!(
                "{} fire rates were detected but there are {} transitions! Aborting...",
                lambdas.len(),
                t_assigns.len()
            );
            exit(0);
        }

        // Get all place names
        for (t_index, t_assign) in t_assigns.enumerate() {
            let both_place_rules = [Rule::input_p, Rule::output_p]
                .into_iter()
                .map(|rule| t_assign.clone().into_inner().find(|r| r.as_rule() == rule))
                .map(|rule_result| rule_result.map(|rule| rule.into_inner()))
                .collect::<Vec<_>>();

            let name: String = t_assign
                .into_inner()
                .find(|r| r.as_rule() == Rule::transition)
                .unwrap()
                .as_str()
                .to_owned();
            let mut input_p_indices: Vec<(usize, usize)> = Vec::new();
            let mut output_p_indices: Vec<(usize, usize)> = Vec::new();

            for (rules_option, indices) in [
                (&both_place_rules[0], &mut input_p_indices),
                (&both_place_rules[1], &mut output_p_indices),
            ] {
                if let Some(rules) = rules_option {
                    for place_rule in rules.clone() {
                        let place_name = place_rule
                            .clone()
                            .into_inner()
                            .find(|pair| pair.as_rule() == Rule::place_name)
                            .unwrap()
                            .as_str();
                        if !all_places.contains(&place_name) {
                            warn!(
                                "Place \"{}\" was not found in set P. Skipping it...",
                                place_name
                            );
                            continue;
                        }
                        let token_result = place_rule
                            .clone()
                            .into_inner()
                            .find(|pair| pair.as_rule() == Rule::tokens);
                        let tokens: usize = match token_result {
                            Some(tokens_rule) => tokens_rule.as_str().parse().unwrap(),
                            None => 1,
                        };
                        let place_index = all_places.iter().position(|s| *s == place_name).unwrap();
                        indices.push((place_index, tokens));
                    }
                }
            }
            transitions.push(Transition {
                transition_id: t_index,
                name,
                pre: input_p_indices,
                succ: output_p_indices,
                fire_rate: lambdas[t_index],
            });
        }

        // If markings and detected places in transition assignments not equal, exit with error msg
        if all_places.len() != initial_marking.len() {
            error!(
                "{} places were detected but initial marking has {} places",
                all_places.len(),
                initial_marking.len()
            );
            error!("Detected places: {:?}", all_places);
            exit(0);
        }

        // Transform place names to actual places
        let places = all_places
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
            initial_marking,
        };
        Box::new(petri_net)
    }
}

fn parse_list<T>(list: &Pairs<Rule>) -> Vec<T>
where
    T: FromStr,
{
    let mut tmp_vec: Vec<T> = Vec::new();
    for rule in list.clone() {
        let input_string = rule.as_str();
        match input_string.parse::<T>() {
            Ok(value) => tmp_vec.push(value),
            Err(_) => panic!("{} is not a valid! Terminating...", input_string),
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
