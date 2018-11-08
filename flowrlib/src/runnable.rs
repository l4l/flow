use serde_json::Value as JsonValue;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use super::implementation::Implementation;

pub type RunnableID = usize;

pub enum RunnableState {
    Init,
    // Runnable has not been initialized yet
    Ready,
    // Runnable's inputs are ready, not blocked on outout - ready to be dispatched
    Blocked,
    // Runnable is blocked from sending output by another runnable
    Waiting,
    // Runnable is waiting for one or more inputs in order to be run
    Running,
    // Runnable has been dispatched and is not running
    Dead,
    // Runnable has run to completion and will not be scheduled again
    Error,
    // There has been some  error in the state transition. Should be impossible to reach
}

pub trait Runnable: RefUnwindSafe + UnwindSafe + Send {
    fn name(&self) -> &str;
    fn id(&self) -> RunnableID;
    fn number_of_inputs(&self) -> usize;
    fn output_destinations(&self) -> &Vec<(&'static str, RunnableID, RunnableID)>;
    fn implementation(&self) -> &Implementation;
    fn get_state(&self) -> &RunnableState;

    fn init(&mut self) -> bool;
    fn write_input(&mut self, input_number: usize, new_value: JsonValue);
    fn input_full(&self, input_number: usize) -> bool;
    fn inputs_ready(&self) -> bool;
    // This runnable has all the inputs necessary and can be run
    fn get_inputs(&mut self) -> Vec<Vec<JsonValue>>;
    fn blocked_on_output(&self) -> bool;
    fn set_state(&mut self, new_state: RunnableState);
}