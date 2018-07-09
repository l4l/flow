use serde_json::Value as JsonValue;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use super::implementation::Implementation;

pub type RunnableID = usize;

pub trait Runnable: RefUnwindSafe + UnwindSafe + Send {
    fn name(&self) -> &str;
    fn id(&self) -> RunnableID;
    fn number_of_inputs(&self) -> usize;
    fn output_destinations(&self) -> &Vec<(&'static str, RunnableID, RunnableID)>;
    fn implementation(&self) -> &Implementation;

    fn init(&mut self) -> bool;
    fn write_input(&mut self, input_number: usize, new_value: JsonValue);
    fn input_full(&self, input_number: usize) -> bool;
    fn inputs_ready(&self) -> bool; // This runnable has all the inputs necessary and can be run
    fn get_inputs(&mut self) -> Vec<Vec<JsonValue>>;
}