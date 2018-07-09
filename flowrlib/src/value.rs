use serde_json::Value as JsonValue;
use runnable::Runnable;
use implementation::Implementation;

const ONLY_INPUT: usize = 0;

pub struct Value<'a> {
    name: String,
    number_of_inputs: usize,
    static_value: bool,
    id: usize,
    initial_value: Option<JsonValue>,
    implementation: &'a Implementation,
    value: JsonValue,
    output_routes: Vec<(&'static str, usize, usize)>,
}

impl<'a> Value<'a> {
    pub fn new(name: &str,
               number_of_inputs: usize,
               static_value: bool,
               _input_depths: Vec<usize>,
               id: usize,
               implementation: &'a Implementation,
               initial_value: Option<JsonValue>,
               output_routes: Vec<(&'static str, usize, usize)>) -> Value<'a> {
        Value {
            name: name.to_string(),
            number_of_inputs,
            static_value,
            id,
            initial_value,
            implementation,
            value: JsonValue::Null,
            output_routes,
        }
    }
}

impl<'a> Runnable for Value<'a> {
    fn name(&self) -> &str { &self.name }
    fn id(&self) -> usize { self.id }
    fn number_of_inputs(&self) -> usize { self.number_of_inputs }
    fn output_destinations(&self) -> &Vec<(&'static str, usize, usize)> { &self.output_routes }
    fn implementation(&self) -> &Implementation { self.implementation }

    /*
        If an initial value is defined then write it to the current value.
        Return true if ready to run as all inputs (single in this case) are satisfied.
    */
    fn init(&mut self) -> bool {
        let value = self.initial_value.clone();
        if let Some(v) = value {
            debug!("\t\tValue initialized by writing '{:?}' to input", &v);
            self.write_input(ONLY_INPUT, v);
        }
        self.inputs_ready()
    }

    /*
        Update the value stored - this should only be called when the value has already been
        consumed by all the listeners and hence it can be overwritten.
    */
    fn write_input(&mut self, _input_number: usize, input_value: JsonValue) {
        self.value = input_value;
    }

    fn input_full(&self, _input_number: usize) -> bool {
        !self.value.is_null()
    }

    // Responds true if all inputs have been satisfied and can be run - false otherwise
    fn inputs_ready(&self) -> bool {
        !self.value.is_null()
    }

    // If the value is a static value then it's value is always available and never get's consumed
    // otherwise, getting the value should consume the value and it will have to get refilled
    fn get_inputs(&mut self) -> Vec<Vec<JsonValue>> {
        if self.static_value {
            vec!(vec!(self.value.clone()))
        } else {
            vec!(vec!(self.value.take()))
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::Value as JsonValue;

    #[test]
    fn destructure_output_base_route() {
        let json = json!("my_value");
        assert_eq!(json.pointer("").unwrap(), "my_value");
    }

    #[test]
    fn destructure_json_value() {
        let json: JsonValue = json!({ "sub_route": "sub_value" });
        assert_eq!(json.pointer("/sub_route").unwrap(), "sub_value");
    }
}