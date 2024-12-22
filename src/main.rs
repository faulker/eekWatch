use eekwatch::{
    commands::{self, checks::exec_rule_check},
    config::load_config,
    log,
    rules::{get_all_rules, get_single_rule},
};
use std::{env, process::exit};

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args[1].to_uppercase();
    let rule = args[2].to_uppercase();

    // Load config
    load_config();

    match action.as_str() {
        "GET" => {
            if rule == "DISKS" {
                commands::disk::display_disks();
            }
        }
        "CHECK" => handle_check_action(&rule),
        _ => log::error("Unknown action".to_string()),
    }
}

fn handle_check_action(rule: &str) {
    if rule == "ALL" {
        // Load all rules
        let rules = match get_all_rules() {
            Ok(rules_list) => rules_list,
            Err(e) => {
                log::error(e);
                exit(1);
            }
        };

        // Run each rule
        for (rule_name, _) in rules {
            match get_single_rule(rule_name) {
                Ok(rc) => exec_rule_check(rc),
                Err(e) => log::error(e),
            }
        }
    } else {
        // Run the specific rule
        match get_single_rule(rule.to_owned()) {
            Ok(rc) => exec_rule_check(rc),
            Err(e) => log::error(e),
        }
    }
}
