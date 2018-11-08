use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Stdout;

impl Implementation for Stdout {
    fn run(&self, _runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, _tx: &Sender<(usize, JsonValue)>) -> RunAgainOption {
        println!("{}", inputs.remove(0).get(0).unwrap().as_str().unwrap());
        RUN_AGAIN
    }
}