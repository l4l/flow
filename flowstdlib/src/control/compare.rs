use serde_json::Value as JsonValue;
use flowrlib::implementation::{Implementation, RunAgain, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Compare;

/*
    A compare operator that takes two numbers (for now) and outputs the comparisons between them
*/
impl Implementation for Compare {
    fn run(&self, runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        let left = inputs[0].remove(0).as_i64().unwrap();
        let right = inputs[1].remove(0).as_i64().unwrap();

        let output = json!({
                    "equal" : left == right,
                    "lt" : left < right,
                    "gt" : left > right,
                    "lte" : left <= right,
                    "gte" : left >= right,
                });
        runnable.send_output(tx, output);

        RUN_AGAIN
    }
}