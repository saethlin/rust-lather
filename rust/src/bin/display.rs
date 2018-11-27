extern crate lather;
extern crate png;
extern crate rayon;

use lather::{linspace, Simulation};
use png::HasParameters;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let sim = Arc::new(Mutex::new(Simulation::sun()));
    linspace(0.0, 100.0, 100)
        .enumerate()
        .collect::<Vec<(usize, f64)>>()
        .par_iter()
        .for_each(|(t, time)| {
            let image = sim.lock().unwrap().draw_rgba(*time);
            save_png(&image, &format!("{:04}.png", t));
        });
}

fn save_png(image: &[u8], filename: &str) {
    let file = ::std::fs::File::create(filename).unwrap();
    let w = &mut ::std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1000, 1000);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(image).unwrap();
}
