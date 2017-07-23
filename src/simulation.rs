extern crate std;
extern crate ini;
extern crate rand;
use std::iter;
use std::sync::RwLock;
use std::fmt::Write;
use self::ini::Ini;
use self::rand::distributions::{IndependentSample, LogNormal, Range};
use std::sync::Arc;
use planck::planck_integral;
use poly_fit_rv::fit_rv;
use compute_bisector::compute_bisector;

use star::Star;
use spot::Spot;

pub struct Observation {
    pub rv: f64,
    pub bisector: Vec<f64>,
}

pub struct Simulation {
    pub star: Arc<Star>,
    pub spots: Vec<Spot>,
    dynamic_fill_factor: f64,
    generator: Arc<RwLock<rand::XorShiftRng>>,
}

pub fn normalize(vector: &mut Vec<f64>) {
    let max = vector.iter().cloned().fold(std::f64::NAN, f64::max);
    for item in vector.iter_mut() {
        *item /= max;
    }
}

macro_rules! get {
(&mut $error:ident, $file:ident, $filename:ident, $section:expr, $field:expr, $type:ty) => (
    { let default_string = "1.0".to_owned();
    $file.section(Some($section)).unwrap().get($field)
        .unwrap_or_else(|| {
            writeln!(
                $error,
                "Missing field {} of section {} in config file {}",
                $field,
                $section,
                $filename).unwrap();
            &default_string
        })
        .parse::<$type>()
        .unwrap_or_else(|_| {
            writeln!(
                $error,
                "Cannot parse field {} of section {} in config file {}",
                $field,
                $section,
                $filename).unwrap();
            <$type as Default>::default()
        })
    }
)
}

impl Simulation {
    pub fn new(filename: &str) -> Simulation {
        let mut error = String::new();
        let file = Ini::load_from_file(filename).expect(&format!(
            "Could not open config file {}",
            filename
        ));

        file.section(Some("star")).expect(&format!(
            "Missing section start in config file {}",
            filename
        ));

        let radius = get!(&mut error, file, filename, "star", "radius", f64);
        let period = get!(&mut error, file, filename, "star", "period", f64);
        let inclination = get!(&mut error, file, filename, "star", "inclination", f64);
        let temperature = get!(&mut error, file, filename, "star", "Tstar", f64);
        let spot_temp_diff = get!(&mut error, file, filename, "star", "Tdiff_spot", f64);
        let limb_linear = get!(&mut error, file, filename, "star", "limb1", f64);
        let limb_quadratic = get!(&mut error, file, filename, "star", "limb2", f64);
        let dynamic_fill_factor = get!(&mut error, file, filename, "star", "fillfactor", f64);
        let grid_size = get!(&mut error, file, filename, "star", "grid_resolution", usize);

        let star = Arc::new(Star::new(
            radius,
            period,
            inclination,
            temperature,
            spot_temp_diff,
            limb_linear,
            limb_quadratic,
            grid_size,
        ));

        let spots: Vec<Spot> = file.iter()
            .filter(|&(s, _)| s.to_owned().is_some())
            .filter(|&(s, _)| s.to_owned().unwrap().as_str().starts_with("spot"))
            .map(|(section, _)| {
                let sec = section.clone().unwrap();
                let latitude = get!(&mut error, file, filename, sec.as_str(), "latitude", f64);
                let longitude = get!(&mut error, file, filename, sec.as_str(), "longitude", f64);
                let size = get!(&mut error, file, filename, sec.as_str(), "size", f64);
                let plage = get!(&mut error, file, filename, sec.as_str(), "plage", bool);

                Spot::new(star.clone(), latitude, longitude, size, plage, false)
            })
            .collect();

        if !error.is_empty() {
            panic!("One or more errors loading config file");
        }

        Simulation {
            star: star,
            spots: spots,
            dynamic_fill_factor: dynamic_fill_factor,
            generator: Arc::new(RwLock::new(rand::XorShiftRng::new_unseeded())),
        }

    }

    fn check_fill_factor(&mut self, time: f64) {
        let mut current_fill_factor = self.spots
            .iter()
            .filter(|s| s.alive(time))
            .map(|s| (s.radius * s.radius) / 2.0)
            .sum::<f64>();

        let fill_range = LogNormal::new(0.5, 4.0);
        let lat_range = Range::new(-30.0, 30.0);
        let long_range = Range::new(0.0, 360.0);

        if current_fill_factor < self.dynamic_fill_factor {
            let mut generator = self.generator.write().expect(
                "Simulation RNG lock was poisoned by another panic",
            );

            while current_fill_factor < self.dynamic_fill_factor {
                let new_fill_factor = iter::repeat(())
                    .map(|_| fill_range.ind_sample(&mut *generator) * 9.4e-6)
                    .find(|v| *v < 0.001)
                    .unwrap();

                let mut new_spot = Spot::new(
                    self.star.clone(),
                    lat_range.ind_sample(&mut *generator),
                    long_range.ind_sample(&mut *generator),
                    new_fill_factor,
                    false,
                    true,
                );
                new_spot.time_appear += time;
                new_spot.time_disappear += time;

                let collides = self.spots
                    .iter()
                    .filter(|s| {
                        s.alive(new_spot.time_appear) || s.alive(new_spot.time_disappear)
                    })
                    .any(|s| new_spot.collides_with(s));

                if !collides {
                    current_fill_factor += (new_spot.radius * new_spot.radius) / 2.0;
                    self.spots.push(new_spot);
                }
            }
        }
    }

    pub fn observe_flux(&mut self, time: &[f64], wavelength_min: f64, wavelength_max: f64)
        -> Vec<f64> {
        let star_intensity = planck_integral(self.star.temperature, wavelength_min, wavelength_max);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength_min, wavelength_max) /
                star_intensity;
        }

        time.iter()
            .map(|t| {
                self.check_fill_factor(*t);
                let spot_flux: f64 = self.spots.iter().map(|s| s.get_flux(*t)).sum();
                (self.star.flux_quiet - spot_flux) / self.star.flux_quiet
            })
            .collect()
    }

    pub fn observe_rv(&mut self, time: &[f64], wavelength_min: f64, wavelength_max: f64)
        -> Vec<Observation> {
        let mut output = Vec::with_capacity(time.len());

        let star_intensity = planck_integral(self.star.temperature, wavelength_min, wavelength_max);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength_min, wavelength_max) /
                star_intensity;
        }

        for t in time.iter() {
            self.check_fill_factor(*t);
        }

        for t in time.iter() {
            let mut spot_profile = vec![0.0; self.star.profile_active.len()];
            for spot in self.spots.iter().filter(|s| s.alive(*t)) {
                let profile = spot.get_ccf(*t);
                for (total, this) in spot_profile.iter_mut().zip(profile.iter()) {
                    *total += *this;
                }
            }

            for (spot, star) in spot_profile.iter_mut().zip(self.star.integrated_ccf.iter()) {
                *spot = *star - *spot;
            }

            let rv = fit_rv(&self.star.profile_quiet.rv, &spot_profile) - self.star.zero_rv;

            let bisector: Vec<f64> = compute_bisector(&self.star.profile_quiet.rv, &spot_profile)
                .iter()
                .map(|b| b - self.star.zero_rv)
                .collect();

            output.push(Observation {
                rv: rv,
                bisector: bisector,
            })
        }
        output
    }
}

impl std::fmt::Debug for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Write;
        let mut message = String::new();
        message.push_str("Star:\n");
        write!(message, "    period: {} d\n", self.star.period)?;
        write!(message, "    inclination: {} rad\n", self.star.inclination)?;
        write!(message, "    temperature: {} K\n", self.star.temperature)?;
        write!(
            message,
            "    spot_temp_diff: {} K\n",
            self.star.spot_temp_diff
        )?;
        write!(message, "    limb_linear: {}\n", self.star.limb_linear)?;
        write!(
            message,
            "    limb_quadratic: {}\n",
            self.star.limb_quadratic
        )?;
        write!(message, "    grid_size: {}\n", self.star.grid_size)?;
        message.push_str("Spots:\n");
        for spot in &self.spots {
            write!(message, "    latitude: {} rad\n", spot.latitude)?;
            write!(message, "    longitude: {} rad\n", spot.longitude)?;
            write!(message, "    radius: {}\n", spot.radius)?;
            write!(message, "    temperature: {} K\n", spot.temperature)?;
            write!(message, "    plage: {}\n", spot.plage)?;
            write!(message, "    mortal: {}\n", spot.mortal)?;
            message.push_str("----");
        }
        let new_len = message.len() - 5;
        message.truncate(new_len);
        f.write_str(message.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config() {
        let sim = Simulation::new("/home/ben/rather/sun.cfg");
        assert_eq!(sim.dynamic_fill_factor, 0.0);
    }
}
