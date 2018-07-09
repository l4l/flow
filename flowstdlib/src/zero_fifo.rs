use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Fifo;

impl Implementation for Fifo {
    fn run(&self, id: usize, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        self.send_output(id, tx, inputs.remove(0).remove(0));
        RUN_AGAIN
    }
}