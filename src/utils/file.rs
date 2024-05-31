use log::error;
use std::{fs, io::ErrorKind, process::exit};

#[allow(dead_code)]
pub fn read_file(path: &str) -> String {
    let file_content = fs::read_to_string(path);
    match file_content {
        Ok(value) => value,
        Err(err_type) => {
            let error_msg: String = match err_type.kind() {
                ErrorKind::NotFound => format!("File \"{}\" not found", path),
                _ => format!("Error occurred while attempting to read {}", path),
            };
            error!("{}", error_msg);
            exit(0);
        }
    }
}

#[derive(Default)]
pub struct TimeMeasurements {
    vwaa_times: Vec<u128>,
    gba_times: Vec<u128>,
    // ba_times: Vec<u128>,
    // pba_times: Vec<u128>,
    // dra_times: Vec<u128>,
    // cross_mdp_times: Vec<u128>,
    // pctl_times: Vec<u128>,
}

impl TimeMeasurements {
    pub fn new() -> Self {
        TimeMeasurements::default()
    }

    pub fn add_time(&mut self, t: usize, microseconds: u128) {
        match t {
            0 => &mut self.vwaa_times,
            1 => &mut self.gba_times,
            // 2 => &mut self.ba_times,
            // 3 => &mut self.pba_times,
            // 4 => &mut self.dra_times,
            // 5 => &mut self.cross_mdp_times,
            // 6 => &mut self.pctl_times,
            _ => unreachable!(),
        }
        .push(microseconds);
    }

    pub fn to_file(&self, folder: &str, suffix: &str, start_idx: usize) {
        let tmp: String = "X Y".into();
        let timing_prefix = vec![
            ("formula_parse_", &self.vwaa_times),
            ("ev_", &self.gba_times),
            // ("ba_", &self.ba_times),
            // ("pba_", &self.pba_times),
            // ("dra_", &self.dra_times),
            // ("cross_mdp_", &self.cross_mdp_times),
            // ("pctl_", &self.pctl_times),
        ];

        for (prefix, timings) in timing_prefix {
            let table = timings
                .iter()
                .enumerate()
                .map(|(n, time)| format!("{} {}", n + start_idx, time))
                .fold("".to_string(), |a, b| a + "\n" + b.as_str());
            let file_name = format!("{}{}{}.txt", folder, prefix, suffix);
            println!("Writing to {}", file_name);
            let _ = fs::write(file_name, tmp.clone() + "\n" + table.as_str());
        }
    }
}
