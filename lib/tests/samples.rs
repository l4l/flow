use std::path::PathBuf;

extern crate flowlib;
use flowlib::loader::loader;
use flowlib::dumper::dump;

#[test]
#[ignore]
fn sample_hello_world_simple() {
    let path = PathBuf::from("../samples/hello-world-simple/context.yaml");
    match loader::load_flow(path) {
        Ok(_) => {},
        _ => assert!(false),
    }
}

#[test]
#[ignore]
fn sample_hello_world() {
    let path = PathBuf::from("../samples/hello-world/context.yaml");
    match loader::load_flow(path) {
        Ok(_) => {},
        _ => assert!(false),
    }
}

#[test]
fn dump_hello_simple_toml() {
    let path = PathBuf::from("../samples/hello-world-simple-toml/context.toml");
    match loader::load_flow(path) {
        Ok(f) => dump(f, 0),
        _ => assert!(false),
    }
}

#[test]
fn dump_hello_world_toml_context() {
    let path = PathBuf::from("../samples/hello-world-toml/context.toml");
    match loader::load_flow(path) {
        Ok(f) => dump(f, 0),
        _ => assert!(false),
    }
}

#[test]
fn dump_hello_world_flow1_toml() {
    let path = PathBuf::from("../samples/hello-world-toml/flow1.toml");
    match loader::load_flow(path) {
        Ok(f) => dump(f, 0),
        _ => assert!(false),
    }
}

#[test]
fn dump_complex1() {
    let path = PathBuf::from("../samples/complex1/context.toml");
    match loader::load_flow(path) {
        Ok(f) => dump(f, 0),
        _ => assert!(false),
    }
}