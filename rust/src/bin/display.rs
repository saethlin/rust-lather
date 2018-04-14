extern crate lather;
extern crate png;
use lather::{linspace, Simulation};

fn main() {
    let mut sim = Simulation::new("sun.cfg");

    for time in linspace(5.0, 25.0, 100) {
        let mut pix_image = sim.star.draw_rgba();
        sim.draw_rgba(time, &mut pix_image);
        save_png(&pix_image, &format!("{:.2}.png", time));
    }
}

fn save_png(image: &[u8], filename: &str) {
    use std::path::Path;
    use std::fs::File;
    use std::io::BufWriter;
    use png::HasParameters;

    let path = Path::new(filename);
    let file = File::create(path).unwrap();
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1000, 1000);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(image).unwrap();
}
