use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct ToString;

impl Implementation for ToString {
    fn run(&self, runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        let input = inputs.remove(0).remove(0);
        match input {
            JsonValue::String(_) => {
                runnable.send_output(tx, input);
            },
            JsonValue::Bool(boolean) => {
                runnable.send_output(tx, JsonValue::String(boolean.to_string()));
            },
            JsonValue::Number(number) => {
                runnable.send_output(tx, JsonValue::String(number.to_string()));
            },
            JsonValue::Array(array) => {
                for entry in array {
                    runnable.send_output(tx,entry);
                }
            },
            _ => {}
        };

        RUN_AGAIN
    }
}