use flowrlib::implementation::Implementation;
use flowrlib::implementation::RunAgain;
use flowrlib::process::Process;
use flowrlib::runlist::RunList;
use num::Complex;
use serde_json::Value as JsonValue;

pub struct CreateComplex;

/*
    Given the row and column of a pixel in the output image, return the
    corresponding point on the complex plane.

    `bounds` is a pair giving the width and height of the image in pixels.
    `pixel` is a (row, column) pair indicating a particular pixel in that image.
    The `upper_left` and `lower_right` parameters are points on the complex
    plane designating the area our image covers.
*/
impl Implementation for CreateComplex {
    fn run(&self, process: &Process, mut inputs: Vec<Vec<JsonValue>>, run_list: &mut RunList) -> RunAgain {
        let arg1 = inputs.remove(0).remove(0);
        let arg2 = inputs.remove(0).remove(0);

        match (arg1, arg2) {
            (JsonValue::Number(re), JsonValue::Number(im)) => {
                let output = json!({ "re" : re, "im": im });
                run_list.send_output(process, output);
            },
            _  => {}
        }

        true
    }
}

/// Take a pair of floating-point numbers and create a complex type
pub fn create_complex(re: f64, im: f64) -> Complex<f64> {
    Complex { re, im }
}

#[cfg(test)]
mod tests {
    use flowrlib::process::Process;
    use flowrlib::runlist::RunList;
    use serde_json::Value as JsonValue;

    use super::CreateComplex;

    #[test]
    fn parse_complex_ok() {
        // Create input args - two floating point numbers
        let arg1 = json!(1.5);
        let arg2 = json!(1.6);

        let inputs: Vec<Vec<JsonValue>> = vec!(vec!(arg1), vec!(arg2));

        let mut run_list = RunList::new();
        let pc = &Function::new("pc", 2, true, vec!(1, 1, 1), 0, Box::new(CreateComplex), None, vec!()) as &Process;
        let implementation = pc.implementation();

        implementation.run(pc, inputs, &mut run_list);
    }
}


