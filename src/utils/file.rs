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
    times: Vec<u128>,
}

impl TimeMeasurements {
    pub fn new() -> Self {
        TimeMeasurements::default()
    }

    pub fn add_time(&mut self, microseconds: u128) {
        self.times.push(microseconds);
    }

    pub fn to_file(&self, file_name: &str, start_idx: usize) {
        let tmp: String = "X Y".into();
        let table = self
            .times
            .iter()
            .enumerate()
            .map(|(n, time)| format!("{} {}", n + start_idx, time))
            .fold("".to_string(), |a, b| a + "\n" + b.as_str());
        let _ = fs::write(file_name, tmp + "\n" + table.as_str());
    }
}
