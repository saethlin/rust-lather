extern crate rather;
extern crate gnuplot;
use gnuplot::{Figure, AxesCommon, Color};

use rather::simulation::Simulation;
use rather::linspace::linspace;

fn main() {
    let time: Vec<f64> = linspace(0.0, 25.05, 10).collect();
    let mut sim = Simulation::new("/home/ben/rather/sun.cfg");

    println!("{:?}", sim);

    let fluxes = sim.observe_flux(&time, 4000e-10, 5000e-10);
    let rv_obs = sim.observe_rv(&time, 4000e-10, 5000e-10);
    let rv_values = rv_obs.iter().map(|obs| obs.rv);

    let mut rv_fig = Figure::new();
    rv_fig.axes2d()
        .lines(&time, rv_values, &[Color("black")])
        .set_title("RV Observations", &[]);
    rv_fig.show();

    let mut flux_fig = Figure::new();
    flux_fig.axes2d()
        .lines(&time, fluxes, &[Color("black")])
        .set_title("Flux Observations", &[]);
    flux_fig.show();


}
