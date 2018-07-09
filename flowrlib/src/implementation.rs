use serde_json::Value as JsonValue;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::sync::mpsc::Sender;
use runlist::OutputSet;

pub type RunAgain = bool;
pub const RUN_AGAIN: RunAgain = true;
pub const DONT_RUN_AGAIN: RunAgain = false;

pub trait Implementation : RefUnwindSafe + UnwindSafe + Sync {
    // An implementation runs, receiving an array of inputs and possibly producing an output
    fn run(&self, id: usize, inputs: Vec<Vec<JsonValue>>, tx: Sender<OutputSet>) -> RunAgain;

    /*
        Create an OutputSet from an implementation that just ran, and send it to the runlist
        server (via the tx channel for that) so it can change the state of runnables in the runlist.
    */
    fn send_output(&self, id: usize, tx: &Sender<OutputSet>, output: JsonValue) {
        tx.send(OutputSet{from: id, output}).unwrap();
    }
}