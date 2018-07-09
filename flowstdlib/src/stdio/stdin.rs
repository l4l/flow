use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use std::io::{self, Read};

pub struct Stdin;

impl Implementation for Stdin {
    fn run(&self, runnable: &Runnable, _inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        let mut buffer = String::new();
        if let Ok(size) = io::stdin().read_to_string(&mut buffer) {
            if size > 0 {
                runnable.send_output(tx, JsonValue::String(buffer.trim().to_string()));
            }
        }

        DONT_RUN_AGAIN
    }
}