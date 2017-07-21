extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use self::test::Bencher;
    use simulation::Simulation;
    use linspace::linspace;

    #[bench]
    fn observe_flux(b: &mut Bencher) {
        let mut sim = Simulation::new("/home/ben/rather/sun.cfg");
        let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

        b.iter(|| sim.observe_flux(&time, 4000e-10, 5000e-10));
    }

    #[bench]
    fn observe_rv(b: &mut Bencher) {
        let mut sim = Simulation::new("/home/ben/rather/sun.cfg");
        let time: Vec<f64> = linspace(0.0, 25.05, 100).collect();

        b.iter(|| sim.observe_rv(&time, 4000e-10, 5000e-10));
    }
}
