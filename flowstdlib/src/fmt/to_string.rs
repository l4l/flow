use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct ToString;

impl Implementation for ToString {
    fn run(&self, runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        let input = inputs.remove(0).remove(0);
        match input {
            JsonValue::String(_) => {
                send_output(runnable.id(), tx, input, true /* done */, run_again: RUN_AGAIN);
            },
            JsonValue::Bool(boolean) => {
                send_output(runnable.id(), tx, JsonValue::String(boolean.to_string()),
                            true /* done */, run_again: RUN_AGAIN);
            },
            JsonValue::Number(number) => {
                send_output(runnable.id(), tx, JsonValue::String(number.to_string()),
                            true /* done */, run_again: RUN_AGAIN);
            },
            JsonValue::Array(array) => {
                for entry in array {
                    send_output(runnable.id(), tx, entry, false /* done */, run_again: RUN_AGAIN);
                    // TODO send true for "done" for the last entry!
                }
            },
            _ => {}
        };
    }
}