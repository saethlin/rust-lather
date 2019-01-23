//! The `lather` crate models starspot effects on photometric
//! and radial velocity observations, with a Python interface.
//!
//! This project was inspired by a desire to improve upon the
//! starspot modeling library named SOAP.
#[cfg(feature = "simd")]
extern crate faster;
extern crate quadrature;
extern crate rand;
extern crate rayon;
extern crate rulinalg;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

mod boundingshape;
mod bounds;
mod distributions;
mod fit_rv;
mod linspace;
mod planck;
mod point;
mod profile;
mod simulation;
mod solar_ccfs;
mod spot;
mod star;

pub use bounds::Bounds;
pub use linspace::{floatrange, linspace};
pub use simulation::Observation;
pub use simulation::Simulation;
pub use spot::SpotConfig;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

/// Build a simulation from a path to a config file
#[no_mangle]
pub unsafe extern "C" fn simulation_new(
    filename: *const c_char,
    error: *mut *const c_char,
) -> *mut Simulation {
    let filename = match CStr::from_ptr(filename).to_str() {
        Ok(v) => v,
        Err(_) => {
            let error_message = CString::new("Filename ust be valid UTF-8").unwrap();
            *error = error_message.as_ptr() as *mut c_char;
            std::mem::forget(error_message);
            return std::ptr::null_mut();
        }
    };

    match Simulation::from_config(&Path::new(filename)) {
        Ok(v) => {
            let obj = Box::new(v);
            Box::into_raw(obj)
        }
        Err(e) => {
            let error_message = CString::new(e).unwrap();
            *error = error_message.as_ptr() as *mut c_char;
            std::mem::forget(error_message);
            std::ptr::null_mut()
        }
    }
}

/// Close down a simulation
#[no_mangle]
pub unsafe extern "C" fn simulation_free(sim: *mut Simulation) {
    if sim.is_null() {
        return;
    }

    // Take ownership of the pointer so that we drop it
    let _: Box<Simulation> = ::std::mem::transmute(sim);
}

/// Print a simulation
#[no_mangle]
pub unsafe extern "C" fn simulation_tostring(sim: *mut Simulation) -> *const c_char {
    use std::fmt::Write;
    if sim.is_null() {
        return std::ptr::null();
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
        return std::ptr::null();
    }
    let time_slice = std::slice::from_raw_parts(times, n_times);
    let output = (*sim).observe_flux(time_slice, Bounds::new(wave_start, wave_end));
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
        return std::ptr::null();
    }

    let time_slice = std::slice::from_raw_parts(times, n_times);
    let observations = (*sim).observe_rv(time_slice, Bounds::new(wave_start, wave_end));
    let mut output = Vec::with_capacity(n_times * 401);
    for ob in &observations {
        output.push(ob.rv);
    }
    for ob in &observations {
        output.extend(&ob.ccf);
    }
    let ptr = output.as_ptr();
    std::mem::forget(output);
    ptr
}

/// Remove all spots on this simulation
#[no_mangle]
pub unsafe extern "C" fn simulation_clear_spots(sim: *mut Simulation) {
    if sim.is_null() {
        return;
    }

    (*sim).clear_spots();
}

/// Add a spot to the simulation
#[no_mangle]
pub unsafe extern "C" fn simulation_add_spot(
    sim: *mut Simulation,
    latitude: f64,
    longitude: f64,
    fill_factor: f64,
    plage: bool,
) {
    if sim.is_null() {
        return;
    }

    (*sim).add_spot(&SpotConfig {
        latitude,
        longitude,
        fill_factor,
        plage: Some(plage),
        temperature: None,
    });
}

#[no_mangle]
pub unsafe extern "C" fn simulation_draw_bgr(sim: *mut Simulation, time: f64, image_ptr: *mut u8) {
    let image = std::slice::from_raw_parts_mut(image_ptr, 1000 * 1000 * 3);
    (*sim).draw_bgr(time, image);
}
