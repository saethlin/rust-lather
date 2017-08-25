//! The `lather` crate models starspot effects on photometric
//! and radial velocity observations, with a Python interface.
//!
//! This project was inspired by a desire to improve upon the
//! starspot modeling library named SOAP.

#[deny(missing_docs)]
#[macro_use]
extern crate cpython;
extern crate numpy;
extern crate ndarray;
extern crate rayon;

mod simulation;
pub use simulation::Simulation;
pub use simulation::Observation;

mod linspace;
pub use linspace::{floatrange, linspace};

mod resolution;
mod bounds;
mod boundingshape;
mod point;
mod fit_rv;
mod planck;
mod compute_bisector;
mod star;
mod spot;
mod profile;
mod sun_ccfs;

#[doc(hidden)]
pub use py_interface::PyInit_lather;

#[allow(missing_docs)]
mod py_interface {
    use std::cell::RefCell;
    use cpython::PyResult;
    use numpy::{IntoPyArray, IntoPyResult, PyArray, PyArrayModule};
    use ndarray::ArrayViewD;
    use simulation::Simulation;

    py_module_initializer!(lather, initlather, PyInit_lather, |py, m| {
        m.add_class::<PySimulation>(py)?;
        Ok(())
    });

    py_class!(class PySimulation |py| {
        data sim: RefCell<Simulation>;

        def __new__(_cls, filename: &str) -> PyResult<PySimulation> {
            PySimulation::create_instance(py, RefCell::new(Simulation::new(filename)))
        }

        def __repr__(&self) ->PyResult<String> {
            Ok(format!("{:?}", self.sim(py).borrow()))
        }

        def __str__(&self) ->PyResult<String> {
            Ok(format!("{:?}", self.sim(py).borrow()))
        }

        def observe_flux(&self, time: PyArray, wavelength_min: f64, wavelength_max: f64) -> PyResult<PyArray> {
            let np = PyArrayModule::import(py)?;
            let time: ArrayViewD<f64> = time.as_array().into_pyresult(py, "time must be an array of f64")?;
            let time_vec: Vec<f64> = time.iter().cloned().collect();
            let flux = self.sim(py).borrow_mut().observe_flux(&time_vec, wavelength_min, wavelength_max);
            Ok(flux.into_pyarray(py, &np))
        }

        def observe_rv(&self, time: PyArray, wavelength_min: f64, wavelength_max: f64) -> PyResult<(PyArray, PyArray)> {
            let np = PyArrayModule::import(py)?;
            let time: ArrayViewD<f64> = time.as_array().into_pyresult(py, "time must be an array of f64")?;
            let time_vec: Vec<f64> = time.iter().cloned().collect();
            let observations = self.sim(py).borrow_mut().observe_rv(&time_vec, wavelength_min, wavelength_max);

            let rv: Vec<f64> = observations.iter().map(|o| o.rv).collect();
            let mut bis_data = Vec::<f64>::with_capacity(rv.len() * observations[0].bisector.len());
            for bisector in observations.iter().map(|o| o.bisector.clone()) {
                bis_data.extend(bisector);
            }
            let bisectors = PyArray::new::<f64>(py, &np, &[rv.len(), observations[0].bisector.len()]);
            for (input, mut output) in bis_data.iter().zip(bisectors.as_slice_mut().unwrap()) {
                *output = *input;
            }

            Ok((rv.into_pyarray(py, &np), bisectors))
        }
    });
}