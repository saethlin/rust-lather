//! The `lather` crate models starspot effects on photometric
//! and radial velocity observations, with a Python interface.
//!
//! This project was inspired by a desire to improve upon the
//! starspot modeling library named SOAP.

#![deny(missing_docs)]
#[macro_use]
extern crate cpython;
#[macro_use]
extern crate derivative;
extern crate dimensioned as dim;
extern crate ini;
extern crate itertools;
extern crate ndarray;
extern crate num_complex;
extern crate numpy;
extern crate quadrature;
extern crate rand;
extern crate rayon;
extern crate rgsl;
extern crate rulinalg;
extern crate rustfft;

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
            Ok(format!("{:#?}", self.sim(py).borrow()))
        }

        def __str__(&self) ->PyResult<String> {
            Ok(format!("{:#?}", self.sim(py).borrow()))
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

            let rv: Vec<_> = observations.iter().map(|o| o.rv.value_unsafe).collect();
            let mut bis_data = Vec::<f64>::with_capacity(rv.len() * observations[0].bisector.len());
            for bisector in observations.iter().map(|o| o.bisector.clone()) {
                bis_data.extend(bisector.iter().map(|b| b.value_unsafe));
            }
            let bisectors = PyArray::new::<f64>(py, &np, &[rv.len(), observations[0].bisector.len()]);
            for (input, output) in bis_data.iter().zip(bisectors.as_slice_mut().unwrap()) {
                *output = *input;
            }

            Ok((rv.into_pyarray(py, &np), bisectors))
        }
    });
}

use dim::traits::Dimensioned;
use std::ops::Neg;

pub trait Abs {
    fn abs(self) -> Self;
}

impl<
    T: Dimensioned<Value = V, Units = U> + Neg<Output = T>,
    V: PartialOrd<f64> + Neg<Output = V>,
    U,
> Abs for T
{
    fn abs(self) -> Self {
        if self.value_unsafe() < &0.0 {
            -self
        } else {
            self
        }
    }
}
