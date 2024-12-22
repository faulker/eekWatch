use std::fs::OpenOptions;

use std::io::prelude::*;

use crate::log;

pub fn write_to_log(file_path: &str, data: &str) {
    match OpenOptions::new().create(true).append(true).open(file_path) {
        Ok(mut file) => {
            writeln!(file, "{}", data);
        }
        Err(e) => log::error(format!("Failed to open log file: {}", e)),
    };
}
