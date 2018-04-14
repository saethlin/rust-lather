//! The `lather` crate models starspot effects on photometric
//! and radial velocity observations, with a Python interface.
//!
//! This project was inspired by a desire to improve upon the
//! starspot modeling library named SOAP.

#[macro_use]
extern crate derivative;
#[cfg(feature = "simd")]
extern crate faster;
extern crate itertools;
extern crate ndarray;
extern crate num_complex;
extern crate quadrature;
extern crate rand;
extern crate rayon;
extern crate rgsl;
extern crate rulinalg;
extern crate rustfft;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod simulation;
pub use simulation::Simulation;
pub use simulation::Observation;
pub use spot::{Spot, SpotConfig};

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

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Build a simulation from a path to a config file
#[no_mangle]
pub unsafe extern "C" fn simulation_new(filename: *const c_char) -> *mut Simulation {
    let mut obj = Box::new(Simulation::from_config(
        CStr::from_ptr(filename).to_str().unwrap(),
    ));

    let ptr: *mut _ = &mut *obj;

    // Forget discards its argument (passed by-move), without triggering its
    // destructor, if it has one.
    ::std::mem::forget(obj);

    ptr
}

/// Close down a simulation
#[no_mangle]
pub unsafe extern "C" fn simulation_free(sim: *mut Simulation) {
    if sim.is_null() {
        return;
    }

    // Now, we know the pointer is non-null, we can continue.
    let obj: Box<Simulation> = ::std::mem::transmute(sim);

    // Explicitly drop the object, unnecessary but nice
    ::std::mem::drop(obj);
}

/// Print a simulation
#[no_mangle]
pub unsafe extern "C" fn simulation_tostring(sim: *mut Simulation) -> *const c_char {
    use std::fmt::Write;
    if sim.is_null() {
        return 0 as *const c_char;
    }

    let mut output = String::new();
    write!(&mut output, "{:#?}", *sim).unwrap();
    let output = CString::new(output).unwrap();
    let ptr = output.as_ptr();
    std::mem::forget(output);
    ptr
}

/// Observe the flux of a simulation at given time values in days
#[no_mangle]
pub unsafe extern "C" fn simulation_observe_flux(
    sim: *mut Simulation,
    times: *mut f64,
    n_times: usize,
    wave_start: f64,
    wave_end: f64,
) -> *const f64 {
    if sim.is_null() {
        return 0 as *const f64;
    }
    let time_slice = std::slice::from_raw_parts(times, n_times);
    let output = (*sim).observe_flux(time_slice, wave_start, wave_end);
    let ptr = output.as_ptr();
    std::mem::forget(output);
    ptr
}

/// Observe the rv and bisectors of a simulation at given time values in days
#[no_mangle]
pub unsafe extern "C" fn simulation_observe_rv(
    sim: *mut Simulation,
    times: *mut f64,
    n_times: usize,
    wave_start: f64,
    wave_end: f64,
) -> *const f64 {
    if sim.is_null() {
        return 0 as *const f64;
    }

    let time_slice = std::slice::from_raw_parts(times, n_times);
    let observations = (*sim).observe_rv(time_slice, wave_start, wave_end);
    let mut output = Vec::with_capacity(n_times * 1001);
    for ob in &observations {
        output.push(ob.rv);
    }
    for ob in &observations {
        output.extend(&ob.bisector);
    }
    let ptr = output.as_ptr();
    std::mem::forget(output);
    ptr
}
