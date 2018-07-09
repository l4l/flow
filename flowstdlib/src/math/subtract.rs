use serde_json;
use serde_json::Value as JsonValue;
use serde_json::Value::Number;
use flowrlib::implementation::{Implementation, RunAgain, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Subtract;

// TODO implementation of `std::ops::Add` might be missing for `&serde_json::Number`

impl Implementation for Subtract {
    fn run(&self, runnable: &Runnable, inputs: Vec<Vec<JsonValue>>, tx: &Sender<(usize, JsonValue)>) -> RunAgain {
        let input_a = inputs.get(0).unwrap();
        let input_b = inputs.get(1).unwrap();
        match (&input_a[0], &input_b[0]) {
            (&Number(ref a), &Number(ref b)) => {
                // TODO mixed signed and unsigned integers
                if a.is_i64() && b.is_i64() {
                    let n = JsonValue::Number(serde_json::Number::from(a.as_i64().unwrap() - b.as_i64().unwrap()));
                    runnable.send_output(tx, n);
                } else if a.is_u64() && b.is_u64() {
                    let n = JsonValue::Number(serde_json::Number::from(a.as_u64().unwrap() - b.as_u64().unwrap()));
                    runnable.send_output(tx, n);
                } else if a.is_f64() && b.is_f64() {
                    let n = JsonValue::Number(serde_json::Number::from_f64(a.as_f64().unwrap() - b.as_f64().unwrap()).unwrap());
                    runnable.send_output(tx, n);
                }
            }
            (_, _) => {}
        }

        RUN_AGAIN
    }
}