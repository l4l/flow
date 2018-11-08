use serde_json::Value as JsonValue;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::sync::mpsc::Sender;
use runlist::OutputSet;

pub enum RunAgainOption { RunAgain, DontRunAgain }

pub trait Implementation : RefUnwindSafe + UnwindSafe + Sync {
    // An implementation runs, receiving an array of inputs and possibly producing an output
    fn run(&self, id: usize, inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>);

    /*
        Create an OutputSet from an implementation that just ran, and send it to the runlist
        server (via the tx channel for that) so it can change the state of runnables in the runlist.
    */
    fn send_output(&self, id: usize, tx: &Sender<OutputSet>, output: JsonValue,
                   done: bool, run_again: RunAgainOption) {
        tx.send(OutputSet{from: id, output, done, run_again}).unwrap();
    }
}