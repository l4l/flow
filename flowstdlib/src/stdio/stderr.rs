use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Stderr;

impl Implementation for Stderr {
    fn run(&self, _runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, _tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        eprintln!("{}", inputs.remove(0).get(0).unwrap().as_str().unwrap());
        RUN_AGAIN
    }
}