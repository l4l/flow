use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Stderr;

impl Implementation for Stderr {
    fn run(&self, _runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, _tx: &Sender<OutputSet>) {
        eprintln!("{}", inputs.remove(0).get(0).unwrap().as_str().unwrap());
        send_output(runnable.id(), tx, JsonNull, true /* done */, RUN_AGAIN);
    }
}