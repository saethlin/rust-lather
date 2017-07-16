extern crate rand;
use std::f64::consts;

use profile::Profile;
use boundingshape::Bounds;
use sun_ccfs::*;

static SOLAR_RADIUS: f64 = 696000.0;
static DAYS_TO_SECONDS: f64 = 86400.0;

pub struct Star {
    period: f64,
    inclination: f64,
    temperature: f64,
    spot_temp_diff: f64,
    limb_linear: f64,
    limb_quadratic: f64,
    grid_interval: f64,
    intensity: f64,
    flux_quiet: f64,
    zero_rv: f64,
    equatorial_velocity: f64,
    integrated_ccf: Vec<f64>,
    fit_result: Vec<f64>,
}

impl Star {
    pub fn new(radius: f64, period: f64, inclination: f64, temperature: f64, spot_temp_diff: f64,
               limb_linear: f64, limb_quadratic: f64, grid_size: usize) {

        let sqrt = f64::sqrt;

        let edge_velocity = (2.0 * consts::PI * radius * SOLAR_RADIUS) / (period * DAYS_TO_SECONDS);
        let equatorial_velocity = edge_velocity * inclination.sin();

        let profile_quiet = Profile::new(rv(), ccf_quiet());
        let profile_active = Profile::new(rv(), ccf_active());

        let (fluxes, weighted_ccfs) = (0..grid_size).map(|a| (a as f64 * (2.0 / grid_size as f64)) - 1.0)
            .map(|y| {
                let shifted_ccf = profile_quiet.shift(y * equatorial_velocity);
                let z_bound = sqrt(1.0 - y*y);
                let limb_integral = limb_integral(Bounds::new(-z_bound, z_bound), y, limb_linear, limb_quadratic);
                return (limb_integral, shifted_ccf.iter().map(|a| a*limb_integral));
            }).unzip();

        let quiet_flux = fluxes.sum();

        /* This is a pretty direct translation from C++ for what I want to do above
        for y in (0..grid_size).map(|a| (a as f64 * (2.0 / grid_size as f64)) - 1.0) {
            let ccf_shifted = profile_quiet.shift(y * equatorial_velocity);
            let z_bound = sqrt(1.0 - y*y);
            let limb_integral = limb_integral(Bounds::new(-z_bound, z_bound), y, limb_linear, limb_quadratic);
            for (ref mut integrated, shifted) in integrated_ccf.iter().zip(ccf_shifted) {
                *integrated = *integrated + (shifted * limb_integral);
            }
            flux_quiet += limb_integral;
        }
        */
    }
}

pub fn min(a: f64, b: f64) -> f64 {
    if a < b {return a;}
    b
}

pub fn limb_integral(z_bounds: Bounds, y: f64, limb_linear: f64, limb_quadratic: f64) -> f64 {
    if z_bounds.lower == z_bounds.upper {
        return 0.0;
    }

    let z_upper = z_bounds.upper;
    let z_lower = z_bounds.lower;

    let x_upper = (1.0 - min(z_upper*z_upper + y*y, 1.0)).sqrt();
    let x_lower = (1.0 - min(z_lower*z_lower + y*y, 1.0)).sqrt();

    1./6. * (z_upper * (3.0*limb_linear*(x_upper-2.0) + 2.0*(limb_quadratic*(3.0*x_upper + 3.0*y*y + z_upper*z_upper - 6.0) + 3.0)) -
            3.0 * (y*y - 1.0)*(limb_linear + 2.0*limb_quadratic)*(z_upper/x_upper).atan())
            - 1./6. * (z_lower * (3.0*limb_linear*(x_lower-2.0) + 2.0*(limb_quadratic*(3.0*x_lower + 3.0*y*y + z_lower*z_lower - 6.0) + 3.0)) -
                                    3.0 * (y*y - 1.0)*(limb_linear + 2.0 *limb_quadratic)*(z_lower/x_lower).atan())
}