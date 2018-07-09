use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use std::io::{self};

pub struct Readline;

impl Implementation for Readline {
    fn run(&self, runnable: &Runnable, _inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n > 0 {
                    runnable.send_output(tx, JsonValue::String(input.trim().to_string()));
                    return true;
                }
            }
            Err(_) => {}
        }

        DONT_RUN_AGAIN
    }
}