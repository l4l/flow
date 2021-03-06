use flowrlib::implementation::Implementation;
use flowrlib::implementation::RunAgain;
use flowrlib::process::Process;
use flowrlib::runlist::RunList;
use image::ColorType;
use image::png::PNGEncoder;
use serde_json::Value as JsonValue;
use std::io::Write;

pub struct FormatPNG;

impl Implementation for FormatPNG {
    fn run(&self, _process: &Process, mut inputs: Vec<Vec<JsonValue>>, _run_list: &mut RunList) -> RunAgain {
        let bytes = inputs.remove(0).remove(0);

        // bounds: (usize, usize),
        let bounds = inputs.remove(0).remove(0);
        let width = bounds["width"].as_u64().unwrap() as u32;
        let height = bounds["height"].as_u64().unwrap() as u32;

        debug!("Writing image of width '{}' and height '{}'", width, height);

        let mut png_buffer = Vec::new();
        let encoder = PNGEncoder::new(png_buffer.by_ref());
        encoder.encode(bytes.as_str().unwrap().as_bytes(), width, height, ColorType::Gray(8))
            .expect("error encoding pixels as PNG");


        // TODO
//        let string = String::from_utf8_lossy(&png_buffer).to_string();
//        run_list.send_output(runnable, JsonValue::String(string));

        true
    }
}
