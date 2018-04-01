use serde_json::Value as JsonValue;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use runnable::Runnable;
use runlist::RunList;

pub trait Implementation : RefUnwindSafe + UnwindSafe {
    // An implementation runs, receiving an array of inputs and possibly producing an output
    fn run(&self, runnable: &Runnable, inputs: Vec<JsonValue>, run_list: &mut RunList);
}