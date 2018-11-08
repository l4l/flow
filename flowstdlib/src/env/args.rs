use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, DONT_RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use std::env;

pub struct Args;

impl Implementation for Args {
    fn run(&self, runnable: &Runnable, _inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        if let Ok(args) = env::var("FLOW_ARGS") {
            env::remove_var("FLOW_ARGS"); // so another invocation later won't use it by mistake
            let flow_args: Vec<&str> = args.split(' ').collect();
            send_output(runnable.id(), tx, json!(flow_args), true /* done */, run_again: DONT_RUN_AGAIN);
        }
    }
}