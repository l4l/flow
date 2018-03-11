use flowrlib::implementation::Implementation;

pub struct Reverse;

impl Implementation for Reverse {
    fn run(&self, mut inputs: Vec<Option<String>>) -> Option<String> {
        let input = inputs.remove(0).unwrap();
        let output = input.chars().rev().collect::<String>();
        Some(output)
    }
}