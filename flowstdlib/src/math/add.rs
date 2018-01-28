use flowrlib::implementation::Implementation;

use std::fmt;
use std::fmt::Debug;

pub struct Add;

const DEFINITION: &'static str ="
name = 'Add'

[[input]]
name = 'i1'
type = 'u32'

[[input]]
name = 'i2'
type = 'u32'

[[output]]
name = 'o1'
type = 'u32'";

impl Implementation for Add {
    fn number_of_inputs(&self) -> usize {
        2
    }

    fn run(&self, inputs: Vec<Option<String>>) -> Option<String> {
        let i1 = inputs[0].clone().unwrap().parse::<i32>().unwrap();
        let i2 = inputs[1].clone().unwrap().parse::<i32>().unwrap();
        let o1 = i1 + i2;
        Some(o1.to_string())
    }

    fn define(&self) -> &'static str {
        DEFINITION
    }

    fn name(&self) -> &'static str {
        "Add"
    }
}

impl Debug for Add {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "add defined in file: '{}'", file!())
    }
}