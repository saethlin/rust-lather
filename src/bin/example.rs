extern crate rather;
extern crate gnuplot;
use gnuplot::*;

fn main() {
    let mut sim = rather::Simulation::new("sun.cfg");
    let time: Vec<f64> = rather::linspace(0.0, 25.05, 100).collect();
    let rv: Vec<f64> = sim.observe_rv(&time, 4000e-10, 5000e-10).iter().map(|o| o.rv).collect();

    let mut fig = Figure::new();
    fig.axes2d()
        .set_x_range(Fix(time[0]), Fix(time.iter().cloned().last().unwrap()))
        .lines(time, rv, &[Color("black")]);
    fig.show();
}
 