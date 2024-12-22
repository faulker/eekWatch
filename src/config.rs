use crate::log;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, env::current_dir, fs, path::PathBuf, process::exit, sync::OnceLock,
};

pub static DEBUG: OnceLock<bool> = OnceLock::new();
pub static RULES_PATH: OnceLock<PathBuf> = OnceLock::new();
pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    debug: Option<bool>,
    pub rules: RulesConf,
    pub alerts: AlertsConf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RulesConf {
    pub location: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertsConf {
    pub email: AlertEmailConf,
    pub logging: AlertLoggingConf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertEmailConf {
    pub smtp: String,
    pub user: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertLoggingConf {
    pub location: String,
    pub file: String,
    pub rotation: HashMap<String, String>,
}

pub fn load_config() {
    // Check if the dev config file exists, fi so use it
    let dev = fs::exists("config-dev.json").unwrap_or(false);
    let config_file = if dev {
        "config-dev.json"
    } else {
        "config.json"
    };

    // Open the config file
    let file = match fs::File::open(config_file) {
        Ok(f) => f,
        Err(e) => {
            log::error(format!("Failed to open config file: {}", e));
            exit(1);
        }
    };

    // Parse the config file
    let reader = std::io::BufReader::new(file);
    let mut cnf: Config = match serde_json::from_reader(reader) {
        Ok(c) => c,
        Err(e) => {
            log::error(format!("Failed to parse config file: {}", e));
            exit(1);
        }
    };

    let path = build_rules_path(cnf.rules.location);
    let debug = cnf.debug.unwrap_or(false);
    cnf.rules.location = path.display().to_string();

    // Load the config into memory for reference in other parts of the application
    CONFIG.get_or_init(|| cnf);

    // Set the debug global flag
    DEBUG.get_or_init(|| debug);

    log::debug("DEBUG flag enabled in config".to_string());
    log::debug(format!("Rules path: {:?}", path));

    let p = RULES_PATH.get_or_init(|| path);
    log::debug(format!("Config rules path: {:?}", p));
}

// Builds the rules path based on the config setting
fn build_rules_path(str_path: String) -> PathBuf {
    let mut path = PathBuf::new();
    let base_path = PathBuf::from(str_path);

    if !base_path.has_root() {
        path.push(current_dir().unwrap());
    }

    path.push(base_path);

    path
}
