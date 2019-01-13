use implementation::Implementation;
use input::Input;
use serde_json::Value as JsonValue;

pub struct Process<'a> {
    name: String,
    number_of_inputs: usize,
    id: usize,
    implementation: &'a Implementation,
    output_routes: Vec<(&'static str, usize, usize)>,

    static_value: bool,
    initial_value: Option<JsonValue>,
    inputs: Vec<Input>,
}

impl<'a> Process<'a> {
    pub fn new(name: &str,
               number_of_inputs: usize,
               static_value: bool,
               input_depths: Vec<usize>,
               id: usize,
               implementation: &'a Implementation,
               initial_value: Option<JsonValue>,
               output_routes: Vec<(&'static str, usize, usize)>) -> Process<'a> {
        let mut process = Process {
            name: name.to_string(),
            number_of_inputs,
            id,
            implementation,
            output_routes,
            static_value, // static value
            initial_value, // initial value
            inputs: Vec::with_capacity(number_of_inputs),
        };

        // Set the correct depths on each input
        for input_depth in input_depths {
            process.inputs.push(Input::new(input_depth));
        }

        process
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn number_of_inputs(&self) -> usize {
        self.number_of_inputs
    }

    pub fn id(&self) -> usize {
        self.id
    }

    /*
        If an initial value is defined then write it to the current value.
        Return true if ready to run as all inputs (single in this case) are satisfied.
    */
    pub fn init(&mut self) -> bool {
        let value = self.initial_value.clone();
        if let Some(v) = value {
            debug!("\t\tValue initialized by writing '{:?}' to input #0", &v);
            self.write_input(0, v);
        }
        self.can_run()
    }

    pub fn write_input(&mut self, input_number: usize, input_value: JsonValue) {
        if !self.inputs[input_number].full() {
            self.inputs[input_number].push(input_value);
        } else {
            // a static value is never emptied when run, so allow it to be overwritten when full
            if self.static_value {
                self.inputs[input_number].overwrite(input_value);
            } else {
                error!("\t\t\tProcess #{} '{}' Input overflow on input number {}", self.id(), self.name(), input_number);
            }
        }
    }

    pub fn output_destinations(&self) -> &Vec<(&'static str, usize, usize)> {
        &self.output_routes
    }

    pub fn implementation(&self) -> &Implementation {
        self.implementation
    }

    pub fn input_full(&self, input_number: usize) -> bool {
        self.inputs[input_number].full()
    }

    // responds true if all inputs have been satisfied and this process can be run - false otherwise
    pub fn can_run(&self) -> bool {
        for input in &self.inputs {
            if !input.full() {
                return false;
            }
        }

        return true;
    }

    pub fn get_input_values(&mut self) -> Vec<Vec<JsonValue>> {
        let mut input_values: Vec<Vec<JsonValue>> = Vec::new();
        for input_value in &mut self.inputs {
            if self.static_value {
                input_values.push(input_value.read());
            } else {
                input_values.push(input_value.take());
            }
        }
        input_values
    }
}

#[cfg(test)]
mod test {
    use serde_json::value::Value as JsonValue;

    use super::Process;
    use super::super::implementation::Implementation;
    use super::super::implementation::RunAgain;
    use super::super::runlist::RunList;

    struct TestImpl;

    impl Implementation for TestImpl {
        fn run(&self, process: &Process, inputs: Vec<Vec<JsonValue>>, run_list: &mut RunList)
               -> RunAgain {
            true
        }
    }

    #[test]
    fn destructure_output_base_route() {
        let json = json!("simple");
        assert_eq!(json.pointer("").unwrap(), "simple");
    }

    #[test]
    fn destructure_json_value() {
        let json: JsonValue = json!({ "sub_route": "sub_output" });
        assert_eq!(json.pointer("/sub_route").unwrap(), "sub_output");
    }

    #[test]
    fn access_array_elements() {
        let args: Vec<&str> = vec!("arg0", "arg1", "arg2");
        let json = json!(args);
        assert_eq!(json.pointer("/0").unwrap(), "arg0");
        assert_eq!(json.pointer("/1").unwrap(), "arg1");
    }

    #[test]
    fn can_send_input_if_empty() {
        let mut process = Process::new("test", 1, false, vec!(1), 0, &TestImpl {}, None, vec!());
        process.init();
        process.write_input(0, json!(1));
        assert_eq!(process.get_input_values().remove(0).remove(0), json!(1));
    }

    #[test]
    fn can_send_input_if_empty_and_static() {
        let mut process = Process::new("test", 1, true, vec!(1), 0, &TestImpl {}, None, vec!());
        process.init();
        process.write_input(0, json!(1));
        assert_eq!(process.get_input_values().remove(0).remove(0), json!(1));
    }

    #[test]
    fn cannot_send_input_if_initialized() {
        let mut process = Process::new("test", 1, false, vec!(1), 0, &TestImpl {},
                                       Some(json!(0)), vec!());
        process.init();
        process.write_input(0, json!(1)); // error
        assert_eq!(process.get_input_values().remove(0).remove(0), json!(0));
    }

    #[test]
    fn can_send_input_if_full_and_static() {
        let mut process = Process::new("test", 1, true, vec!(1), 0, &TestImpl {},
                                       None, vec!());
        process.init();
        process.write_input(0, json!(1));
        process.write_input(0, json!(2));
        assert_eq!(process.get_input_values().remove(0).remove(0), json!(2));
    }

    #[test]
    fn cannot_send_input_if_full_and_not_static() {
        let mut process = Process::new("test", 1, false, vec!(1), 0, &TestImpl {},
                                       None, vec!());
        process.init();
        process.write_input(0, json!(1)); // success
        process.write_input(0, json!(2)); // fail
        assert_eq!(process.get_input_values().remove(0).remove(0), json!(1));
    }
}