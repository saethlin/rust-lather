extern crate rather;
extern crate gnuplot;

fn main() {
    let mut sim = rather::simulation::Simulation::new("/home/ben/rather/sun.cfg");
    let time: Vec<f64> = rather::linspace::linspace(5.9, 7.0, 1000).collect();
    let rv: Vec<f64> = sim.observe_rv(&time, 4000e-10, 5000e-10).iter().map(|o| o.rv).collect();

    use gnuplot::*;

    let mut fig = Figure::new();
    fig.axes2d()
        .set_x_range(Fix(time[0]), Fix(time.iter().cloned().last().unwrap()))
        .lines(time, rv, &[Color("black")]);
    fig.show();
}
 