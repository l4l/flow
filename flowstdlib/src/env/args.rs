use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use std::env;

pub struct Args;

impl Implementation for Args {
    fn run(&self, runnable: &Runnable, _inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        if let Ok(args) = env::var("FLOW_ARGS") {
            env::remove_var("FLOW_ARGS"); // so another invocation later won't use it by mistake
            let flow_args: Vec<&str> = args.split(' ').collect();
            runnable.send_output(tx, json!(flow_args));
        }

        DONT_RUN_AGAIN
    }
}