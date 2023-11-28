mod mcsp;
mod parser;
mod utils;

use crate::mcsp::ModelCheckInfo;
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Path of the input file
    #[arg(short, long)]
    input_file: String,

    /// Max error used by the value iteration algorithm to compute the states satisfying an
    /// 'UNTIL' pctl statement.
    #[arg(long("max-error"), default_value_t = 0.01)]
    max_error: f64
}

fn main() {
    init();
    let args = Args::parse();
    let mc_info = ModelCheckInfo::parse(&args.input_file, args.max_error);
    mc_info.evaluate_pctl();
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
