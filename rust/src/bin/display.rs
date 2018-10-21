extern crate lather;
extern crate png;
use lather::{linspace, Simulation};
use png::HasParameters;

fn main() {
    let mut sim = Simulation::sun();
    for (t, time) in linspace(0.0, 100.0, 100).enumerate() {
        save_png(&sim.draw_rgba(time), &format!("{:04}.png", t));
    }
}

fn save_png(image: &[u8], filename: &str) {
    let file = ::std::fs::File::create(filename).unwrap();
    let w = &mut ::std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1000, 1000);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(image).unwrap();
}
