use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use std::io::{self, Read};

pub struct Stdin;

impl Implementation for Stdin {
    fn run(&self, runnable: &Runnable, _inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        let mut buffer = String::new();
        if let Ok(size) = io::stdin().read_to_string(&mut buffer) {
            if size > 0 {
                send_output(runnable.id(), tx, JsonValue::String(buffer.trim().to_string()),
                            true /* done */, DONT_RUN_AGAIN);
            }
        }
        send_output(runnable.id(), tx, JsonNull, true /* done */, DONT_RUN_AGAIN);
    }
}