use std::fs::File;
use std::path::PathBuf;
use std::io;
use flowrlib::implementation::{Implementation, RunAgainOption, RUN_AGAIN};
use flowrlib::runnable::Runnable;
use std::sync::mpsc::Sender;
use image::png::PNGEncoder;
use image::ColorType;
use flowrlib::runlist::OutputSet;

pub struct WriteBitmap;

impl Implementation for WriteBitmap {
    fn run(&self, _runnable: &Runnable, mut inputs: Vec<Vec<JsonValue>>, _tx: &Sender<OutputSet>) {
        let filename = inputs.remove(0).remove(0);
        let bytes = inputs.remove(0).remove(0);
        let bounds = inputs.remove(0).remove(0);

        // bounds: (usize, usize),
        let width = bounds["width"].as_u64().unwrap() as usize;
        let height = bounds["height"].as_u64().unwrap() as usize;

        debug!("Writing image of width '{}' and height '{}' to file: '{}'",
            width, height, filename);
        write_bitmap(&PathBuf::from(filename.as_str().unwrap()),
                     bytes.as_str().unwrap().as_bytes(),
                     (width, height)).unwrap();

        // TODO how to ask it to run again if no output :-(
    }
}

/// Write the buffer 'pixels', whose dimensions are given by 'bounds', to the file named 'filename'
fn write_bitmap(filename: &PathBuf, pixels: &[u8], bounds: (usize, usize)) -> io::Result<()> {
    let output = File::create(filename).unwrap();

    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))
}