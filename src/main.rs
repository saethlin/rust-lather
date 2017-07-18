
#![feature(alloc_system)]
extern crate alloc_system;

extern crate rather;
extern crate gnuplot;
use gnuplot::{Figure, AxesCommon, Color, PointSymbol};

use rather::simulation::Simulation;
use rather::linspace::linspace;

fn main() {
    //let time: Vec<f64> = linspace(6.5, 8.0, 100).collect();
    //let time: Vec<f64> = linspace(0.0, 25.05, 20).collect();
    let time = [6.5];
    let mut sim = Simulation::new("/home/ben/rather/sun.cfg");

    //println!("Running Simulation\n{:?}", sim);

    let fluxes = sim.observe_flux(&time, 4000e-10, 5000e-10);
    //let rv_obs = sim.observe_rv(&time, 4000e-10, 5000e-10);
    //let rv_values: Vec<f64> = rv_obs.iter().map(|obs| obs.rv).collect();

    println!("{:?}", fluxes);
    //println!("{:?}", rv_values);

    /*
    let mut rv_fig = Figure::new();
    rv_fig.axes2d()
        .lines(&time, rv_values, &[Color("black")])
        .set_title("RV Observations", &[]);
    rv_fig.show();

    let mut flux_fig = Figure::new();
    flux_fig.axes2d()
        .points(&time, fluxes, &[Color("black"), PointSymbol('O')])
        .set_title("Flux Observations", &[]);
    flux_fig.show();
    */
}
