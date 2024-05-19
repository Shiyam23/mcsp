mod common;
mod input_graph;
mod logic;
mod mcsp;
mod parser;
mod test;
mod utils;

use crate::input_graph::pnet::PetriNet;
use crate::input_graph::InputGraphType;
use crate::mcsp::ModelCheck;
use crate::parser::dpn_parser::DPetriNetParser;
use crate::parser::petri_net_parser::PetriNetParser;
use crate::{input_graph::dpnet::DPetriNet, test::PN1};
use clap::Parser;
use utils::file::TimeMeasurements;

#[derive(Parser, Clone)]
pub struct Args {
    /// Path of the input file
    #[arg(short, long, default_value = "")]
    input_file: String,

    /// Max error used by the value iteration algorithm to compute the
    /// 'UNTIL' pctl statement. Must be greater than 0
    #[arg(long("max-error"), default_value_t = 0.01)]
    max_error: f64,

    #[arg(short, long, default_value_t, value_enum)]
    graph_type: InputGraphType,

    #[arg(short, long, default_value_t, value_enum)]
    logic_type: LogicType,

    #[arg(short, long("precision-digits"), default_value_t = 2)]
    precision_digits: i32,

    #[arg(short, long("show-graph"), default_value_t = false)]
    show_graph: bool,
}
#[derive(clap::ValueEnum, Clone, Default)]
pub enum LogicType {
    #[default]
    Pctl,
    LTL,
}

#[allow(unreachable_code, unused_variables)]
fn main() {
    init();
    let mut args = Args::parse();
    args.logic_type = LogicType::Pctl;
    args.graph_type = InputGraphType::DecisionPetri;
    let mut time_m = TimeMeasurements::new();
    for i in 1..=50 {
        println!("{}", i);
        let input = PN1::get_input(i);
        match args.graph_type {
            InputGraphType::Petri => {
                ModelCheck::<PetriNet, PetriNetParser>::start(args.clone(), input, &mut time_m)
            }
            InputGraphType::DecisionPetri => {
                ModelCheck::<DPetriNet, DPetriNetParser>::start(args.clone(), input, &mut time_m)
            }
        };
    }
    time_m.to_file("/Users/shiyam/Desktop/ma/ch_op/graph_conv_dpn1.txt", 1);
}

fn init() {
    env_logger::builder()
        .default_format()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init()
}
