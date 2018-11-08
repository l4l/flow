use serde_json;
use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct ToNumber;

impl Implementation for ToNumber {
    fn run(&self, runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        let input = inputs.remove(0).remove(0);

        match input {
            JsonValue::String(string) => {
                if let Ok(number) = string.parse::<i64>() {
                    let number = JsonValue::Number(serde_json::Number::from(number));
                    send_output(runnable.id(), tx, number, true /* done */, run_again: RUN_AGAIN);
                }
            },
            _ => {}
        };
    }
}