use log::error;
use std::{fs, io::ErrorKind, process::exit};

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
