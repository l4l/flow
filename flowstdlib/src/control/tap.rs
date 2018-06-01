use serde_json::Value as JsonValue;
use flowrlib::implementation::Implementation;
use flowrlib::implementation::RunAgain;
use flowrlib::runlist::RunList;
use flowrlib::runnable::Runnable;

pub struct Tap;

/*
    A control switch function that outputs the "data" input IF the "control" input is true,
    otherwise it does not produce any output
*/
impl Implementation for Tap {
    fn run(&self, runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, run_list: &mut RunList) -> RunAgain {
        let data = inputs[0].remove(0);
        let control = inputs[1].remove(0).as_bool().unwrap();
        if control {
            run_list.send_output(runnable, data);
        }
        true
    }
}