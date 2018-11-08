use serde_json;
use flowrlib::runlist::OutputSet;
use serde_json::Value::Number;
use serde_json::Value::String;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Add;

// TODO implementation of `std::ops::Add` might be missing for `&serde_json::Number`

impl Implementation for Add {
    fn run(&self, runnable: &Runnable, inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        let input_a = inputs.get(0).unwrap();
        let input_b = inputs.get(1).unwrap();
        match (&input_a[0], &input_b[0]) {
            (&Number(ref a), &Number(ref b)) => {
                // TODO mixed signed and unsigned integers
                if a.is_i64() && b.is_i64() {
                    let n = JsonValue::Number(serde_json::Number::from(a.as_i64().unwrap() + b.as_i64().unwrap()));
                    send_output(runnable.id(), tx, n, true /* done */, RUN_AGAIN);
                } else if a.is_u64() && b.is_u64() {
                    let n = JsonValue::Number(serde_json::Number::from(a.as_u64().unwrap() + b.as_u64().unwrap()));
                    send_output(runnable.id(), tx, n, true /* done */, RUN_AGAIN);
                } else if a.is_f64() && b.is_f64() {
                    let n = JsonValue::Number(serde_json::Number::from_f64(a.as_f64().unwrap() + b.as_f64().unwrap()).unwrap());
                    send_output(runnable.id(), tx, n, true /* done */, RUN_AGAIN);
                }
            }
            (&String(ref a), &String(ref b)) => {
                let i1 = a.parse::<i32>().unwrap();
                let i2 = b.parse::<i32>().unwrap();
                let o1 = i1 + i2;
                send_output(runnable.id(), tx, JsonValue::String(o1.to_string()),
                            true /* done */, RUN_AGAIN);
            }
            (_, _) => {
                send_output(runnable.id(), tx, JsonNull, true /* done */, RUN_AGAIN);
            }
        }
    }
}