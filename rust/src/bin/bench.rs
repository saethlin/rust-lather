#[macro_use]
extern crate bencher;
extern crate lather;
use bencher::Bencher;
use lather::{linspace, Bounds, Simulation, SpotConfig};

fn load_config(b: &mut Bencher) {
    b.iter(|| Simulation::from_config("../examples/sun.toml"));
}

fn observe_flux(b: &mut Bencher) {
    let mut sim = Simulation::sun();
    sim.add_spot(&SpotConfig {
        latitude: 30.0,
        longitude: 180.0,
        fill_factor: 0.01,
        plage: false,
    });
    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

    b.iter(|| sim.observe_flux(&time, Bounds::new(4000e-10, 5000e-10)));
}

fn observe_rv(b: &mut Bencher) {
    let mut sim = Simulation::sun();
    sim.add_spot(&SpotConfig {
        latitude: 30.0,
        longitude: 180.0,
        fill_factor: 0.01,
        plage: false,
    });
    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

    b.iter(|| sim.observe_rv(&time, Bounds::new(4000e-10, 5000e-10)));
}

benchmark_group!(benches, observe_rv); //load_config, observe_flux, observe_rv);
benchmark_main!(benches);
