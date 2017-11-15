extern crate glob;

extern crate clap;
use clap::{App, Arg};

extern crate flowlib;
use flowlib::info;
use flowlib::parser::parser;
use flowlib::parser::parser::Result;

mod files;

use std::env;
use std::path::PathBuf;

#[macro_use]
extern crate log;
extern crate log4rs;

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    info!("Logging started using 'log4rs', see log.yaml for configuration details");
    info!("'{}' version {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("'flowlib' version {}", info::version());

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("check")
            .short("c")
            .help("Check the flow only, don't execute it"))
        .arg(Arg::with_name("flow")
            .help("the name of the 'flow' file")
            .required(false)
            .index(1))
        .get_matches();

    // get the file name from the command line, use CDW if it is not present
    let path = match matches.value_of("flow") {
        None => {
            info!("No path specified, so using Current Working Directory");
            env::current_dir().unwrap()
        },
        Some(p) => PathBuf::from(p),
    };

    match files::open(path) {
        Ok(file) => {
            info!("Attempting to load file: '{:?}'", file);

            match parser::load(file) {
                Result::ContextLoaded(context) => {
                    info!("'{}' context parsed and validated correctly", context.name);
                    if !matches.is_present("check") {
                        // run it
                    }
                },
                Result::FlowLoaded(flow) => {
                    info!("'{}' flow parsed and validated correctly", flow.name);
                    println!("Flow loaded successfully, but only Context can be run");
                },
                Result::Error(e) => {
                    println!("{}", e);
                }
            }
        },
        Err(e) => println!("{}", e)
    }
}