use crate::alerts::email::alert;
use crate::rules::Alert;
use crate::{log, rules::RuleConfig};
use regex::Regex;
use std::collections::HashMap;
use sysinfo::{Disk, Disks};

/// Disk space struct
/// Mostly used to make it easier to test
struct DiskSpace {
    total: f64,
    available: f64,
}

struct Limit {
    amount: f64,
    limit_type: String,
}

/// Handle the disk check
pub fn handle_disk_check(rule_details: RuleConfig) {
    log::info(format!("Running Rule: {}", rule_details.name));
    let disk_info = Disks::new_with_refreshed_list();
    let rule_name = &rule_details.name;
    let mut failures: Vec<Vec<String>> = Vec::new();

    // Loop through the rules
    for rule in rule_details.rules {
        match get_disk_info(&rule["disk"], &disk_info) {
            Some(disk) => {
                let disk_info = DiskSpace {
                    total: disk.total_space() as f64,
                    available: disk.available_space() as f64,
                };

                // Convert available space to MB
                let human_available = disk_info.available / 1024.0 / 1024.0;
                let human_total = disk_info.total / 1024.0 / 1024.0;

                if check_space(&rule["option"], &disk_info, &rule["limit"]) {
                    let msg = vec!(
                        format!("Rule '{rule_name}' failed for mount point '{}'", rule["disk"]),
                        format!("Total/Free Space: {} MB/{} MB", human_total.round(), human_available.round()),
                        format!("Warning Limit: {} {}", rule["limit"], rule["option"])
                    );


                    failures.push(msg);
                } else {
                    log::info(format!("Rule '{}' Passed for mount point '{}'", rule_name, rule["disk"]));
                }
            }
            None => {
                log::error("Failed".to_string());
            }
        }
    }

    if failures.len() > 0 {
        handle_alerts(failures, rule_name, &rule_details.alerts);
    }
}

/// Handle alerts if there are any failures
fn handle_alerts(failure_msgs: Vec<Vec<String>>, rule_name: &String, alerts: &HashMap<String, Alert>) {
    let mut html_formated_msgs: Vec<String> = Vec::new();

    for failure_msg in failure_msgs {
        log::fail(failure_msg.join(" - "));

        // Formats all the failure messages for email
        html_formated_msgs.push(failure_msg.join("<br />"));
    }

    if alerts.contains_key("email") {
        let email = alerts.get("email").unwrap();
        let contacts = &email.contacts;
        alert(rule_name, html_formated_msgs.join("<br /><hr /><br />"), contacts);
    }
}

/// Check the disk space
fn check_space(option: &String, disk: &DiskSpace, limit: &String) -> bool {
    match parse_limit(limit) {
        Some(l) => {
            // option can be free or used
            // free = it will check to see if the available free space is with in the limit
            // used = it will check to see if the used space is less then what is currently being used
            match option.as_str() {
                "free" => {
                    return free_check(&l, disk);
                }
                "used" => {
                    return used_check(&l, disk);
                }
                _ => {
                    log::fail(format!("Unknown disk check type '{}'", option));
                }
            }

            return false;
        }
        None => {
            log::fail("Failed to parse rule for disk size check".to_string());
            return false;
        }
    };
}

/// Check the free space
fn free_check(limit: &Limit, disk: &DiskSpace) -> bool {
    let available = disk.available;

    if limit.limit_type.eq("%") {
        let amount_dec = limit.amount / 100.0;
        let min_ava = disk.total * amount_dec;
        return min_ava > available;
    }

    let limit_type = size_conversion(&limit.limit_type);
    // 0 means no conversion option found, this means that the rule doesn't have
    // a size limit with a KB, MB, GB, or TB definition such as "200MB".
    return if limit_type > 0 {
        let convert_size = u64::pow(1024, limit_type) as f64;
        let min_ava = limit.amount * convert_size;
        min_ava > available
    } else {
        log::fail("Failed to parse rule for disk size check".to_string());
        false
    };
}

/// Check the used space
fn used_check(limit: &Limit, disk: &DiskSpace) -> bool {
    let total = disk.total;
    let available = disk.available;
    let used_total = total - available;

    if limit.limit_type.eq("%") {
        let amount_dec = limit.amount / 100.0;
        let used_max = total * amount_dec;
        log::debug(format!("Total used: {} MB, Used max: {} MB", (used_total / 1024f64 / 1024f64), (used_max / 1024f64 / 1024f64)));
        return used_total > used_max;
    }

    let limit_type = size_conversion(&limit.limit_type);
    // 0 means no conversion option found, this means that the rule doesn't have
    // a size limit with a KB, MB, GB, or TB definition such as "200MB".
    return if limit_type > 0 {
        let convert_size = u64::pow(1024, limit_type) as f64;
        let used_max = limit.amount * convert_size;
        log::debug(format!("Total used: {}, Used max: {}", used_total, used_max));
        used_total > used_max
    } else {
        log::fail("Failed to parse rule for disk size check".to_string());
        false
    };
}

/// Convert the size string to a number
fn size_conversion(size_string: &String) -> u32 {
    let size_conversion_map: HashMap<String, u32> = HashMap::from([
        ("KB".to_string(), 1),
        ("MB".to_string(), 2),
        ("GB".to_string(), 3),
        ("TB".to_string(), 4),
    ]);

    if size_conversion_map.contains_key(size_string) {
        return size_conversion_map[size_string];
    }

    return 0;
}

/// Get the disk info defined in the rule
fn get_disk_info<'a>(disk_mount: &'a String, disks: &'a Disks) -> Option<&'a Disk> {
    for disk in disks {
        if disk.mount_point().to_str().unwrap().eq(disk_mount) {
            log::debug(format!(
                "Checking disk: {:?}, {:?}",
                disk.name(),
                disk.mount_point()
            ));
            let correct_disk = disk;
            return Some(correct_disk);
        }
    }

    None
}

/// Parse the limit string
fn parse_limit(limit: &String) -> Option<Limit> {
    let rx = match Regex::new(r"^([\d\.]*)\s?(\D{1,2})?$") {
        Ok(re) => re,
        Err(e) => {
            log::error(format!("Issue with regex: {}", e));
            return None;
        }
    };

    match rx.captures(&limit) {
        Some(cap) => {
            if cap.get(2).is_some() {
                let amount = cap[1].to_owned().parse::<f64>().unwrap();
                let limit_type = cap[2].to_owned().parse::<String>().unwrap().to_uppercase();

                return Some(Limit { amount, limit_type });
            }

            return None;
        }
        None => {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_limit() {
        // Check size indicator
        let limit = "200 MB".to_string();
        let limit_parsed = parse_limit(&limit).unwrap();
        assert_eq!(limit_parsed.amount, 200.0);
        assert_eq!(limit_parsed.limit_type, "MB".to_string());

        // Check percentage
        let limit = "20%".to_string();
        let limit_parsed = parse_limit(&limit).unwrap();
        assert_eq!(limit_parsed.amount, 20.0);
        assert_eq!(limit_parsed.limit_type, "%".to_string());

        // Check no size indicator (should fail)
        let limit = "100".to_string();
        assert!(parse_limit(&limit).is_none());
    }

    #[test]
    fn test_size_conversion() {
        let kb = "KB".to_string();
        let mb = "MB".to_string();
        let gb = "GB".to_string();
        let tb = "TB".to_string();

        assert_eq!(size_conversion(&kb), 1);
        assert_eq!(size_conversion(&mb), 2);
        assert_eq!(size_conversion(&gb), 3);
        assert_eq!(size_conversion(&tb), 4);
    }

    #[test]
    fn test_used_check_percentage() {
        // 75% used
        let disk = DiskSpace {
            total: 100.0,
            available: 15.0,
        };

        // used space greater than 80% will fail
        // this should pass
        let limit_good = Limit {
            amount: 90.0,
            limit_type: "%".to_string(),
        };

        // used space greater than 40% will fail
        // this should fail
        let limit_bad = Limit {
            amount: 40.0,
            limit_type: "%".to_string(),
        };

        // expect that less than 85% has been used
        assert!(!used_check(&limit_good, &disk));

        // expect that more than 75% has been used
        assert!(used_check(&limit_bad, &disk));
    }

    #[test]
    fn test_free_check_percentage() {
        let disk = DiskSpace {
            total: 100.0,
            available: 85.0,
        };

        let limit_good = Limit {
            amount: 80.0, // 80% limit
            limit_type: "%".to_string(),
        };

        let limit_bad = Limit {
            amount: 90.0, // 90% limit
            limit_type: "%".to_string(),
        };

        // expect that there is enough space free on the disk
        assert!(!free_check(&limit_good, &disk));

        // expect that there is not enough space free on the disk
        assert!(free_check(&limit_bad, &disk));
    }
}
