extern crate lather;
extern crate png;
use lather::{linspace, Simulation, SpotConfig};

fn main() {
    let mut sim = Simulation::sun();
    sim.add_spot(&SpotConfig {
        latitude: 30.0,
        longitude: 180.0,
        fill_factor: 0.01,
        plage: false,
    });
    sim.add_spot(&SpotConfig {
        latitude: -30.0,
        longitude: 180.0,
        fill_factor: 0.01,
        plage: false,
    });

    for time in linspace(5.0, 25.0, 100) {
        save_png(&sim.draw_rgba(time), &format!("{:.2}.png", time));
    }
}

fn save_png(image: &[u8], filename: &str) {
    use png::HasParameters;
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;

    let path = Path::new(filename);
    let file = File::create(path).unwrap();
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1000, 1000);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(image).unwrap();
}
