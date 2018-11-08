use flowrlib::runlist::OutputSet;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;

pub struct Divide;

impl Implementation for Divide {
    fn run(&self, runnable: &Runnable, inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
        let dividend = inputs.get(0).unwrap()[0].as_f64().unwrap();
        let divisor = inputs.get(1).unwrap()[0].as_f64().unwrap();

        let output = json!({"dividend:": dividend, "divisor": divisor, "result": dividend/divisor, "remainder": dividend % divisor});
        runnable.send_output(tx, );
        send_output(runnable.id(), tx, data, output /* done */, RUN_AGAIN);
    }
}

#[cfg(test)]
mod test {
    use flowrlib::runnable::Runnable;
    use flowrlib::function::Function;
    use flowrlib::runlist::OutputSet;
    use super::Divide;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;

    #[test]
    fn test_divide() {
        // Create input vector
        let dividend = json!(99);
        let divisor = json!(3);
        let inputs: Vec<Vec<JsonValue>> = vec!(vec!(dividend), vec!(divisor));
        let (tx, _rx): (Sender<OutputSet>, Receiver<(usize, JsonValue)>) = mpsc::channel();

        let d = &Function::new("d", 3, true, vec!(1, 1, 1), 0, &Divide, None, vec!()) as &Runnable;
        let implementation = d.implementation();

        implementation.run(d, inputs, tx);
    }
}