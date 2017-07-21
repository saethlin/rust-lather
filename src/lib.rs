#![feature(test)]
#[macro_use] extern crate cpython;

pub mod bench;
pub mod star;
pub mod spot;
pub mod profile;
pub mod bounds;
pub mod boundingshape;
pub mod point;
pub mod sun_ccfs;
pub mod fit_rv;
pub mod simulation;
pub mod planck;
pub mod compute_bisector;
pub mod linspace;

use std::cell::RefCell;
use cpython::PyResult;
use simulation::Simulation as RustSimulation;

py_module_initializer!(_rather, init_rather, PyInit__rather, |py, m| {
    m.add_class::<Simulation>(py)?;
    Ok(())
});

py_class!(class Simulation |py| {
    data sim: RefCell<RustSimulation>;
    def __new__(_cls, filename: &str) -> PyResult<Simulation> {
        Simulation::create_instance(py, RefCell::new(RustSimulation::new(filename)))
    }

    def __repr__(&self) ->PyResult<String> {
        Ok(format!("{:?}", self.sim(py).borrow()))
    }

    def __str__(&self) ->PyResult<String> {
        Ok(format!("{:?}", self.sim(py).borrow()))
    }

    def observe_flux(&self, time: Vec<f64>, wavelength_min: f64, wavelength_max: f64) -> PyResult<Vec<f64>> {
        Ok(self.sim(py).borrow_mut().observe_flux(&time, wavelength_min, wavelength_max))
    }

    def observe_rv(&self, time: Vec<f64>, wavelength_min: f64, wavelength_max: f64) -> PyResult<Vec<(f64, Vec<f64>)>> {
        let obs_vec = self.sim(py).borrow_mut().observe_rv(&time, wavelength_min, wavelength_max);
        Ok(obs_vec.iter().map(|o| (o.rv, o.bisector.clone())).collect())
    }
});
