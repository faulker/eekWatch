use crate::{config, log};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleConfig {
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub rules: Vec<HashMap<String, String>>,
    pub alerts: HashMap<String, Alert>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Alert {
    pub contacts: Vec<String>,
}

/// Get a single rule's details
pub fn get_single_rule(rule_name: String) -> Result<RuleConfig, String> {
    let rules_path = config::RULES_PATH.get().unwrap();
    if !rules_path.exists() {
        return Err("Rules path doesn't exist, check the config file!".to_string());
    }

    let rule_path = rules_path.join(format!("{}.json", rule_name));
    log::debug(format!("Rule path: {}", rule_path.display()));

    if !rule_path.exists() {
        return Err(format!(
            "Rule doesn't exist, rule path: {}",
            rule_path.display()
        ));
    }

    let rule_details = get_rule_data(&rule_path).unwrap();

    Ok(rule_details)
}

/// Get details for all rules
pub fn get_all_rules() -> Result<HashMap<String, RuleConfig>, String> {
    let mut rules: HashMap<String, RuleConfig> = HashMap::new();

    match get_rules_list() {
        Ok(file_list) => {
            for rule_path in file_list {
                let f_name = String::from(rule_path.file_name().unwrap().to_string_lossy())
                    .replace(".json", "");
                log::debug(format!(
                    "Rules path: {}, file name: {}",
                    rule_path.display(),
                    f_name
                ));

                let rule_details: RuleConfig = get_rule_data(&rule_path).unwrap();

                rules.insert(f_name, rule_details);
            }
        }
        Err(e) => log::error(e),
    }

    Ok(rules)
}

/// Load the rule file and parse the data
fn get_rule_data(rule_path: &PathBuf) -> Result<RuleConfig, String> {
    return match std::fs::File::open(rule_path) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(rule) => Ok(rule),
                Err(e) => {
                    log::debug(e.to_string());
                    return Err(format!("Failed to read rules file: {:?}", rule_path));
                }
            }
        }
        Err(_) => {
            return Err(format!(
                "Failed to open rules file for reading: {:?}",
                rule_path
            ));
        }
    };
}

/// Get a list of the file listed in the rules path
fn get_rules_list() -> Result<Vec<PathBuf>, String> {
    let rules_path = config::RULES_PATH.get().unwrap();
    if !rules_path.exists() {
        return Err("Rules path doesn't exist, check the config file!".to_string());
    }

    let rules = match fs::read_dir(rules_path) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            // Map the directory entries to paths
            .map(|dir_entry| dir_entry.path())
            .filter_map(|path| {
                if path.extension().map_or(false, |ext| ext == "json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(_) => {
            return Err("Failed to read plugin directory".to_string());
        }
    };

    Ok(rules)
}
