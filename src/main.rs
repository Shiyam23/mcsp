mod mcsp;
mod utils;
use crate::mcsp::ModelCheckInfo;

fn main() {
    init();
    let mc_info = ModelCheckInfo::parse("examples/petri_net.txt", "examples/pctl_test.txt");
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
