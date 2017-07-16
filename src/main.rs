extern crate rather;
use rather::simulation::Simulation;

fn main() {
    let mut sim = Simulation::new("/home/ben/rather/sun.cfg");
    let fluxes = sim.observe_flux(vec![0.0, 1.0, 2.0], 4000e-10, 5000e-10);
    for f in fluxes {
        println!("{:?}", f);
    }
    let rv_obs = sim.observe_rv(vec![0.0, 1.0, 2.0], 4000e-10, 5000e-10);
    for obs in rv_obs {
        println!("{:?}", obs.rv);
    }
}
