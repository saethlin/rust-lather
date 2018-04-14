//! The `lather` crate models starspot effects on photometric
//! and radial velocity observations, with a Python interface.
//!
//! This project was inspired by a desire to improve upon the
//! starspot modeling library named SOAP.

#![deny(missing_docs)]
#[macro_use]
extern crate derivative;
#[cfg(feature = "simd")]
extern crate faster;
extern crate ini;
extern crate itertools;
extern crate ndarray;
extern crate num_complex;
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

/// Test function
#[no_mangle]
pub unsafe extern "C" fn a_function_from_rust() -> i32 {
    42
}

