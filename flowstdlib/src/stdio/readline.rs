use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use std::io::{self};

pub struct Readline;

impl Implementation for Readline {
    fn run(&self, runnable: &Runnable, _inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n > 0 {
                    send_output(runnable.id(), tx, JsonValue::String(input.trim().to_string()),
                                true /* done */, RUN_AGAIN);
                } else {
                    send_output(runnable.id(), tx, JsonNull, true /* done */, RUN_AGAIN);
                }
            }
            Err(_) => {
                send_output(runnable.id(), tx, JsonNull, true /* done */, RUN_AGAIN);
            }
        }
    }
}