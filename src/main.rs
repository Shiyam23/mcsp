mod mcsp;
mod utils;
mod parser;

use clap::Parser;
use crate::mcsp::ModelCheckInfo;

#[derive(Parser)]
struct Args {
    /// Path of the input file
    #[arg(short, long)]
    input_file: String,
}

fn main() {
    init();
    let args = Args::parse();
    let mc_info = ModelCheckInfo::parse(&args.input_file);
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
