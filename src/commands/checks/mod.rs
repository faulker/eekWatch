use crate::rules::RuleConfig;
use disk::handle_disk_check;

pub mod disk;

/// Check types
/// - disk
/// - cpu
/// - memory
pub fn exec_rule_check(rule: RuleConfig) {
    // determine rule
    match rule.rule_type.as_str() {
        "disk" => handle_disk_check(rule),
        "cpu" => todo!("cpu"),
        _ => todo!("test"),
    }
    // process rule logic
}
