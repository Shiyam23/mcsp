use std::collections::HashSet;
use std::convert::Into;
use petgraph::graph::NodeIndex;
use crate::logic::pctl::PctlImpl;
use crate::LogicType;
use crate::mcsp::PctlInfo;

pub mod pctl;

const FORMULA_ID: &str = "PHI";

pub trait LogicImpl {

    fn parse(&self, content: &str) -> Box<dyn Formula>;

    fn find_formula(&self, content: &str) -> Option<String>{
        if let Some(start_index) = content.find(FORMULA_ID){
            let end_index = start_index + FORMULA_ID.len();
            let logic_sub_string: String = content.get(start_index..).unwrap().into();
            return match content[end_index..].find(FORMULA_ID) {
                None => Some(logic_sub_string),
                Some(_) => None,
            }
        }
        None
    }
}

pub fn parse_formula(logic_type: LogicType, content: &str) -> Box<dyn Formula>{
    match logic_type {
        LogicType::Pctl => PctlImpl.parse(content),
        LogicType::LTL => todo!()
    }
}

pub trait Formula {
    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex>;
}