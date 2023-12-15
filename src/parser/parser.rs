use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::{collections::HashMap, str::FromStr};

use crate::parser::mc_parser;
use crate::pctl::StatePhi;
use crate::utils::file::read_file;

// All identifiers for the input petri net
const PLACE_ID: &str = "P";
const TRANSITIONS_ID: &str = "T";
const INPUT_ARCS_ID: &str = "I";
const OUTPUT_ARCS_ID: &str = "O";
const INITIAL_MARKINGS_ID: &str = "M";
const LAMBDAS_ID: &str = "L";

// All identifiers for the input model checking settings
const AP_MAP_ID: &str = "AP_MAP";
const PHI_ID: &str = "PHI";

const PETRI_TAGS: [&str; 6] = [
    PLACE_ID,
    TRANSITIONS_ID,
    INPUT_ARCS_ID,
    OUTPUT_ARCS_ID,
    INITIAL_MARKINGS_ID,
    LAMBDAS_ID,
];

type Marking = Vec<usize>;
type Markings = Vec<Marking>;
type ApMap = HashMap<String, Markings>;

#[derive(Parser)]
#[grammar = "parser/petri_net.pest"]
struct PetriNetParser;

pub struct InputData {
    pub petri_net: PetriNetInfo,
    pub ap_map: ApMap,
    pub phi: Box<dyn StatePhi>,
}
#[derive(Default, Debug)]
pub struct PetriNetInfo {
    pub places: Vec<String>,
    pub transitions: Vec<String>,
    pub input_arcs: Vec<(String, String)>,
    pub output_arcs: Vec<(String, String)>,
    pub initial_marking: Vec<usize>,
    pub lambdas: Vec<f64>,
}

struct ParseErrorWithSource<T>
where
    T: FromStr,
{
    source: String,
    #[allow(dead_code)]
    error: T::Err,
}

pub fn parse(file_path: &str) -> InputData {
    let content = read_file(file_path);
    let petri_net = PetriNetParser::parse(Rule::Main, &content).unwrap();

    // Get all petri net tuples and elements
    let mut petri_elements: HashMap<String, Vec<String>> = HashMap::new();
    let mut petri_pairs: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for petri_tag in PETRI_TAGS {
        let tag_string = match petri_net.find_first_tagged(petri_tag) {
            Some(tag_string) => tag_string,
            None => panic!(),
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
    let petri_net_info: PetriNetInfo = initialize(petri_elements, petri_pairs);
    validate(&petri_net_info);

    let ap_map_rule: Pair<Rule> = petri_net.find_first_tagged(AP_MAP_ID).unwrap();
    let ap_map: ApMap = mc_parser::transform_ap_map(&ap_map_rule);

    let phi_rule: Pair<Rule> = petri_net.find_first_tagged(PHI_ID).unwrap();
    let phi = mc_parser::transform_state(&phi_rule);

    InputData {
        petri_net: petri_net_info,
        ap_map,
        phi,
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
    let mut petri_net_info: PetriNetInfo = PetriNetInfo::default();
    petri_net_info.places = petri_elements.get(PLACE_ID).unwrap().clone();
    petri_net_info.transitions = petri_elements.get(TRANSITIONS_ID).unwrap().clone();
    petri_net_info.input_arcs = petri_pairs.get(INPUT_ARCS_ID).unwrap().clone();
    petri_net_info.output_arcs = petri_pairs.get(OUTPUT_ARCS_ID).unwrap().clone();
    petri_net_info.initial_marking =
        match parse_list(petri_elements.get(INITIAL_MARKINGS_ID).unwrap()) {
            Ok(list) => list,
            Err(e) => panic!("{} is not a valid number! Terminating...", e.source),
        };
    petri_net_info.lambdas = match parse_list(petri_elements.get(LAMBDAS_ID).unwrap()) {
        Ok(list) => list,
        Err(e) => panic!("{} is not a valid decimal number! Terminating...", e.source),
    };
    petri_net_info
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
