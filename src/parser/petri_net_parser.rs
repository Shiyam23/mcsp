use std::collections::HashMap;
use std::str::FromStr;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use crate::input_graph::{ApMap, ParseImpl};
use crate::input_graph::pnet::{Marking, PetriNet, Place, Transition};
use crate::utils::common::ParseErrorWithSource;
use crate::utils::common::ParseOrQuit;

const TRANSITIONS_ID: &str = "T";
const INPUT_ARCS_ID: &str = "I";
const OUTPUT_ARCS_ID: &str = "O";
const INITIAL_MARKINGS_ID: &str = "M";
const PLACE_ID: &str = "P";
const LAMBDAS_ID: &str = "L";
const AP_MAP_ID: &str = "AP_MAP";
const PETRI_TAGS: [&str; 6] = [
    PLACE_ID,
    TRANSITIONS_ID,
    INPUT_ARCS_ID,
    OUTPUT_ARCS_ID,
    INITIAL_MARKINGS_ID,
    LAMBDAS_ID,
];

#[derive(Parser)]
#[grammar = "parser/petri_net.pest"]
pub struct InputParser;

#[derive(Default, Debug)]
pub struct PetriNetInfo {
    pub places: Vec<String>,
    pub transitions: Vec<String>,
    pub input_arcs: Vec<(String, String)>,
    pub output_arcs: Vec<(String, String)>,
    pub initial_marking: Vec<usize>,
    pub lambdas: Vec<f64>,
}

pub struct PetriNetParser;

impl ParseImpl<PetriNet> for PetriNetParser {
    fn parse(content: &str) -> Box<PetriNet> {
        let pairs = InputParser::parse(Rule::Main, content).unwrap();
        let mut petri_elements: HashMap<String, Vec<String>> = HashMap::new();
        let mut petri_pairs: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for petri_tag in PETRI_TAGS {
            let tag_string = match pairs.find_first_tagged(petri_tag) {
                Some(tag_string) => tag_string,
                None => panic!("{} not found even though the input file was parsed successfully", petri_tag),
            };
            match petri_tag {
                PLACE_ID | TRANSITIONS_ID | INITIAL_MARKINGS_ID | LAMBDAS_ID => {
                    let set_element = tag_string
                        .into_inner()
                        .map(|p| p.as_str().to_string())
                        .collect();
                    petri_elements.insert(petri_tag.to_string(), set_element);
                }
                INPUT_ARCS_ID | OUTPUT_ARCS_ID => {
                    let mut set_of_pairs: Vec<(String, String)> = Vec::new();
                    for token in tag_string.into_inner() {
                        let pair_elements: Vec<&str> = token.into_inner().map(|e| e.as_str()).collect();
                        set_of_pairs.push((pair_elements[0].to_string(), pair_elements[1].to_string()))
                    }
                    petri_pairs.insert(petri_tag.to_string(), set_of_pairs);
                }
                &_ => unreachable!(),
            }
        }

        let p_info: PetriNetInfo = initialize(petri_elements, petri_pairs);
        validate(&p_info);
        let places: Vec<Place> = p_info
            .places
            .iter()
            .enumerate()
            .map(|(index, place)| Place {
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
                    places
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
                    places
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
        let ap_map_pairs = pairs.find_first_tagged(AP_MAP_ID).unwrap();
        let ap_map = transform_ap_map(ap_map_pairs);
        let petri_net = PetriNet {
            places,
            transitions,
            ap_map,
            initial_marking: p_info.initial_marking
        };
        Box::new(petri_net)
    }
}

fn validate(petri_net_info: &PetriNetInfo) {
    let states = &petri_net_info.places;
    let transitions = &petri_net_info.transitions;
    let input_arcs = &petri_net_info.input_arcs;
    let output_arcs = &petri_net_info.output_arcs;
    let initial_markings = &petri_net_info.initial_marking;
    let lambdas = &petri_net_info.lambdas;

    // States and Transitions cant share the same name
    if let Some(s) = states.iter().find(|s| transitions.contains(s)) {
        panic!(
            "{} can not be state and transition at the same time! Terminating ...",
            s
        );
    }

    // Check if number of markings are the same as states
    if states.len() != initial_markings.len() {
        panic!(
            "There are {} states but {} initial markings! Terminating ...",
            transitions.len(),
            initial_markings.len()
        );
    }

    // Check if number of lambdas are the same as transitions
    if transitions.len() != lambdas.len() {
        panic!(
            "There are {} transitions but {} lambdas! Terminating ...",
            transitions.len(),
            lambdas.len()
        );
    }

    // Checking input arcs
    let (input_states, input_transitions): (Vec<_>, Vec<_>) = input_arcs.iter().cloned().unzip();
    if let Some(s) = input_states.iter().find(|s| !states.contains(s)) {
        panic!(
            "'{}' is part of an input arc but is not a state! Terminating...",
            s
        );
    }
    if let Some(t) = input_transitions.iter().find(|t| !transitions.contains(t)) {
        panic!(
            "'{}' is part of an input arc but is not a transition! Terminating ...",
            t
        );
    }

    // Checking output arcs
    let (output_transitions, output_states): (Vec<_>, Vec<_>) = output_arcs.iter().cloned().unzip();
    if let Some(s) = output_states.iter().find(|s| !states.contains(s)) {
        panic!(
            "'{}' is part of an output arc but is not a state! Terminating...",
            s
        );
    }
    if let Some(t) = output_transitions.iter().find(|t| !transitions.contains(t)) {
        panic!(
            "'{}' is part of an output arc but is not a transition! Terminating ...",
            t
        );
    }
}

fn initialize(
    petri_elements: HashMap<String, Vec<String>>,
    petri_pairs: HashMap<String, Vec<(String, String)>>,
) -> PetriNetInfo {
    let places = petri_elements.get(PLACE_ID).unwrap().clone();
    let transitions = petri_elements.get(TRANSITIONS_ID).unwrap().clone();
    let input_arcs = petri_pairs.get(INPUT_ARCS_ID).unwrap().clone();
    let output_arcs = petri_pairs.get(OUTPUT_ARCS_ID).unwrap().clone();
    let initial_marking =
        match parse_list(petri_elements.get(INITIAL_MARKINGS_ID).unwrap()) {
            Ok(list) => list,
            Err(e) => panic!("{} is not a valid number! Terminating...", e.source),
        };
    let lambdas = match parse_list(petri_elements.get(LAMBDAS_ID).unwrap()) {
        Ok(list) => list,
        Err(e) => panic!("{} is not a valid decimal number! Terminating...", e.source),
    };
    PetriNetInfo {
        places,
        transitions,
        input_arcs,
        output_arcs,
        initial_marking,
        lambdas
    }
}

fn parse_list<T>(list: &Vec<String>) -> Result<Vec<T>, ParseErrorWithSource<T>>
    where
        T: FromStr,
{
    let mut tmp_vec = Vec::new();
    for value_as_string in list {
        match value_as_string.parse::<T>() {
            Ok(value) => tmp_vec.push(value),
            Err(e) => {
                return Err(ParseErrorWithSource {
                    source: value_as_string.to_owned(),
                    error: e,
                })
            }
        }
    }
    Ok(tmp_vec)
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