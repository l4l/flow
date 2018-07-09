use serde_json;
use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct ToNumber;

impl Implementation for ToNumber {
    fn run(&self, runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        let input = inputs.remove(0).remove(0);

        match input {
            JsonValue::String(string) => {
                if let Ok(number) = string.parse::<i64>() {
                    let number = JsonValue::Number(serde_json::Number::from(number));
                    runnable.send_output(tx, number);
                }
            },
            _ => {}
        };

        RUN_AGAIN
    }
}