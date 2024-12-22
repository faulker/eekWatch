use crate::config;
use std::borrow::Borrow;

#[derive(Debug)]
enum LogTypes {
    Info,
    Warn,
    Error,
    Fail,
}

pub fn debug(msg: String) {
    if *config::DEBUG.get_or_init(|| false) {
        println!("DEBUG: {:#?}", msg);
    }
}

pub fn info<B: Borrow<String>>(msg: B) {
    print_msg(LogTypes::Info, msg);
}
pub fn warn<B: Borrow<String>>(msg: B) {
    print_msg(LogTypes::Warn, msg);
}
pub fn error<B: Borrow<String>>(msg: B) {
    print_msg(LogTypes::Error, msg);
}
pub fn fail<B: Borrow<String>>(msg: B) {
    print_msg(LogTypes::Fail, msg);
}

fn print_msg<B: Borrow<String>>(t: LogTypes, msg: B) {
    let pre: String = match t {
        LogTypes::Info => "INFO: ".to_string(),
        LogTypes::Warn => "WARN: ".to_string(),
        LogTypes::Error => "ERROR: ".to_string(),
        LogTypes::Fail => "FAIL: ".to_string(),
    };

    println!("{}{}", pre, msg.borrow());
}
