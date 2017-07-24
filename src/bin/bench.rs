#[macro_use] extern crate bencher;
use bencher::Bencher;
extern crate rather;
use rather::simulation::Simulation;
use rather::linspace::linspace;

fn create_sim(b: &mut Bencher) {
    b.iter(|| Simulation::new("/home/ben/rather/sun.cfg"));
}

fn observe_flux(b: &mut Bencher) {
    let mut sim = Simulation::new("/home/ben/rather/sun.cfg");
    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

    b.iter(|| sim.observe_flux(&time, 4000e-10, 5000e-10));
}

fn observe_rv(b: &mut Bencher) {
    let mut sim = Simulation::new("/home/ben/rather/sun.cfg");
    let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

    b.iter(|| sim.observe_rv(&time, 4000e-10, 5000e-10));
}

fn bench_bisector(b: &mut Bencher) {
    use rather::compute_bisector::compute_bisector;
    let sim = Simulation::new("/home/ben/rather/sun.cfg");

    b.iter(|| compute_bisector(&sim.star.profile_quiet.rv, &sim.star.integrated_ccf));
}

fn bench_fit_rv(b: &mut Bencher) {
    use rather::poly_fit_rv::fit_rv;
    let sim = Simulation::new("/home/ben/rather/sun.cfg");

    b.iter(|| fit_rv(&sim.star.profile_quiet.rv, &sim.star.integrated_ccf))
}

fn bench_profile_shift(b: &mut Bencher) {
    use rather::profile::Profile;
    use rather::sun_ccfs::*;
    let profile = Profile::new(rv(), ccf_active());

    b.iter(|| profile.shift(1.0));
}

fn bench_quadrature_planck_integral(b: &mut Bencher) {
    use rather::planck::planck_integral;
    b.iter(|| planck_integral(5778.0, 4000e-10, 7000e-10));
}

benchmark_group!(benches, create_sim, observe_flux, observe_rv, bench_bisector,
    bench_fit_rv, bench_profile_shift, bench_quadrature_planck_integral);
benchmark_main!(benches);
