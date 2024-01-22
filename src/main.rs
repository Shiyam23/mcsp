mod mcsp;
mod utils;
mod logic;
mod input_graph;
mod parser;

use crate::mcsp::ModelCheck;
use clap::Parser;
use log::info;
use crate::input_graph::dpnet::DPetriNet;
use crate::input_graph::InputGraphType;
use crate::input_graph::pnet::PetriNet;
use crate::parser::dpn_parser::DPetriNetParser;
use crate::parser::petri_net_parser::PetriNetParser;

#[derive(Parser)]
pub struct Args {
    /// Path of the input file
    #[arg(short, long)]
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
    show_graph: bool

}
#[derive(clap::ValueEnum, Clone, Default)]
pub enum LogicType {
    #[default]
    Pctl,
    LTL
}

fn main() {
    init();
    let args = Args::parse();
    info!("Starting MCSP...");
    match args.graph_type {
        InputGraphType::Petri => ModelCheck::<PetriNet, PetriNetParser>::start(args),
        InputGraphType::DecisionPetri => ModelCheck::<DPetriNet, DPetriNetParser>::start(args)
    };
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
