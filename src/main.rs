#![feature(alloc_system)]
extern crate alloc_system;

extern crate rather;
extern crate gnuplot;
use gnuplot::{AxesCommon, Color, Figure, PointSymbol};

use rather::simulation::Simulation;
use rather::linspace::linspace;

fn main() {
    let do_flux = false;
    let do_rv = true;
    let plots = false;

    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();
    let mut sim = Simulation::new("/home/ben/rather/sun.cfg");
    println!("Running Simulation\n{:?}", sim);

    if do_flux {
        let fluxes = sim.observe_flux(&time, 4000e-10, 5000e-10);
        if plots {
            let mut flux_fig = Figure::new();
            flux_fig
                .axes2d()
                .points(&time, fluxes, &[Color("black"), PointSymbol('O')])
                .set_title("Flux Observations", &[]);
            flux_fig.show();
        } else {
            println!("{:?}", fluxes);
        }
    }

    if do_rv {
        let rv_obs = sim.observe_rv(&time, 4000e-10, 5000e-10);
        let rv_values: Vec<f64> = rv_obs.iter().map(|obs| obs.rv).collect();
        if plots {
            let mut rv_fig = Figure::new();
            rv_fig
                .axes2d()
                .points(&time, rv_values, &[Color("black"), PointSymbol('O')])
                .set_title("RV Observations", &[]);
            rv_fig.show();
        } else {
            for r in rv_values.iter() {
                println!("{}", r);
            }
        }
    }
}
