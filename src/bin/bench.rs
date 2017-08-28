#[macro_use]
extern crate bencher;
use bencher::Bencher;
extern crate lather;
use lather::{Simulation, linspace};

fn create_sim(b: &mut Bencher) {
    b.iter(|| Simulation::new("sun.cfg"));
}

fn observe_flux(b: &mut Bencher) {
    let mut sim = Simulation::new("sun.cfg");
    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

    b.iter(|| sim.observe_flux(&time, 4000e-10, 5000e-10));
}

fn observe_rv(b: &mut Bencher) {
    let mut sim = Simulation::new("sun.cfg");
    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

    b.iter(|| sim.observe_rv(&time, 4000e-10, 5000e-10));
}

fn draw_star(b: &mut Bencher) {
    let sim = Simulation::new("sun.cfg");
    b.iter(|| sim.star.draw_rgba());
}

fn draw_simulation(b: &mut Bencher) {
    let mut sim = Simulation::new("sun.cfg");
    let mut image = vec![0; 1000 * 1000 * 4];
    b.iter(|| sim.draw_rgba(10.0, &mut image));
}

benchmark_group!(
    benches,
    create_sim,
    observe_flux,
    observe_rv,
    draw_star,
    draw_simulation
);
benchmark_main!(benches);
