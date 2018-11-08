use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Fifo;

impl Implementation for Fifo {
    fn run(&self, id: usize, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        send_output(runnable.id(), tx, inputs.remove(0).remove(0), true /* done */, RUN_AGAIN);
    }
}