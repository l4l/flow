extern crate glob;
use glob::glob;

use std::fs::metadata;
use std::process;
use std::env;

extern crate flow;
//use flow::parser::parser;
use flow::info;

#[macro_use]
extern crate log;
extern crate log4rs;

use std::default::Default;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn print_usage(program: &str) {
    error!("Usage: {} [FILENAME|DIRNAME]", program);
}

fn check(path: &str) {
    info!("Checking file: '{}'", path);

    /* TODO re enable this when can get lib to compile
    match parser::load(&path, true) {
        parser::Result::ContextLoaded(context) => info!("'{}' context parsed and validated correctly", context.name),
        parser::Result::FlowLoaded(flow) => info!("'{}' flow parsed and validated correctly", flow.name),
        parser::Result::Error(error) => error!("{}", error),
        parser::Result::Valid => error!("Unexpected parser failure"),
    }
    */
}

fn find_default_file(dir: &str) -> Option<String> {
    let file_pattern = format!("{}/*.context", dir);
    info!("Looking for files matching: {}", file_pattern);

    for path in glob(file_pattern.as_ref()).unwrap().filter_map(Result::ok) {
        // return first
        match path.to_str() {
            None => return None,
            Some(s) => return Some(s.to_string()),
        }
    }
    return None;
}

fn main() {
    log4rs::init_file("log.toml", Default::default()).unwrap();
    info!("Logging started using 'log4rs', see log.toml for configuration details");

    println!("Flow 'check' version: {}", VERSION);
    println!("Flow Library Version {}", info::version());

    let args: Vec<String> = env::args().collect();

    match args.len() {
        // no arguments passed
        1 => {
            let dir = env::current_dir().unwrap();
            find_default_file(dir.to_str().unwrap()).map( | file | {
                check(&file);
            });
        },

        // one argument passed
        2 => {
            if  metadata(&args[1]).unwrap().is_dir() {
                find_default_file(&args[1]).map( | file | {
                    check(&file);
                });
            } else {
                check(&args[1]);
            }
        },

        _ => {
            print_usage(&args[0]);
            process::exit(-1);
        },
    }
}

#[cfg(test)]
mod tests {
    use super::print_usage;

    #[test]
    fn can_print_usage() {
        print_usage("test");
    }
}
