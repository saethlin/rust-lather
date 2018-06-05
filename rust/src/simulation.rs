use compute_bisector::compute_bisector;
use fit_rv::fit_rv;
use planck::planck_integral;
use rand::distributions::{IndependentSample, LogNormal, Range};
use rayon::prelude::*;
use std::iter;
use std::sync::Arc;
use std::sync::RwLock;

use bounds::Bounds;
use spot::Mortality::Mortal;
use spot::{Spot, SpotConfig};
use star::{Star, StarConfig};

/// An observed radial velocity and line bisector.
pub struct Observation {
    /// The radial velocity value in m/s.
    pub rv: f64,
    /// The line bisector in m/s.
    pub bisector: Vec<f64>,
}

/// A model of a star with spots that can be observed.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Simulation {
    star: Arc<Star>,
    spots: Vec<Spot>,
    #[derivative(Debug = "ignore")]
    generator: Arc<RwLock<::rand::StdRng>>,
}

#[derive(Deserialize, Serialize)]
struct Config {
    star: StarConfig,
    spots: Option<Vec<SpotConfig>>,
}

impl Config {
    fn example() -> Config {
        Config {
            star: StarConfig {
                grid_size: 1000,
                radius: 1.0,
                period: 25.05,
                inclination: 90.0,
                temperature: 5778.0,
                spot_temp_diff: 663.0,
                limb_linear: 0.29,
                limb_quadratic: 0.34,
                minimum_fill_factor: 0.00,
            },
            spots: Some(vec![
                SpotConfig {
                    latitude: 30.0,
                    longitude: 180.0,
                    fill_factor: 0.01,
                    plage: false,
                },
                SpotConfig {
                    latitude: 30.0,
                    longitude: 180.0,
                    fill_factor: 0.01,
                    plage: false,
                },
            ]),
        }
    }
}

impl Simulation {
    /// Construct the simulation used in tests
    pub fn sun() -> Simulation {
        Simulation {
            star: Arc::new(Star::from_config(&StarConfig {
                grid_size: 1000,
                radius: 1.0,
                period: 25.05,
                inclination: 90.0,
                temperature: 5778.0,
                spot_temp_diff: 663.0,
                limb_linear: 0.29,
                limb_quadratic: 0.34,
                minimum_fill_factor: 0.00,
            })),
            spots: Vec::new(),
            generator: Arc::new(RwLock::new(::rand::StdRng::new().unwrap())),
        }
    }

    /// Construct a new Star from a TOML file.
    pub fn from_config(config_path: &str) -> Simulation {
        use std::fs::File;
        use std::io::Read;

        let mut contents = String::new();
        File::open(&config_path)
            .unwrap_or_else(|_| {
                panic!(
                    "Tried to open a config file at {:?}, but it doesn't seem to exist",
                    config_path
                );
            })
            .read_to_string(&mut contents)
            .unwrap_or_else(|_| {
                panic!("Unable to read config file at {:?}", &config_path);
            });

        let config: Config = ::toml::from_str(&contents).unwrap_or_else(|_| {
            println!(
                "{:?} is not a valid config file. Here's an example:\n",
                &config_path
            );
            println!("{}", ::toml::to_string_pretty(&Config::example()).unwrap());
            panic!();
        });

        let mut sim = Simulation {
            star: Arc::new(Star::from_config(&config.star)),
            spots: Vec::new(),
            generator: Arc::new(RwLock::new(::rand::StdRng::new().unwrap())),
        };

        if let Some(spot_configs) = config.spots {
            for spot_config in spot_configs {
                sim.spots
                    .push(Spot::from_config(Arc::clone(&sim.star), &spot_config));
            }
        }

        sim
    }

    pub fn add_spot(&mut self, config: &SpotConfig) {
        self.spots
            .push(Spot::from_config(Arc::clone(&self.star), config));
    }

    pub fn clear_spots(&mut self) {
        self.spots.clear();
    }

    fn check_fill_factor(&mut self, time: f64) {
        let mut current_fill_factor = self
            .spots
            .iter()
            .filter(|s| s.alive(time))
            .map(|s| (s.radius * s.radius) / 2.0)
            .sum::<f64>();

        let fill_range = LogNormal::new(0.5, 4.0);
        let lat_range = Range::new(-30.0, 30.0);
        let long_range = Range::new(0.0, 360.0);

        if current_fill_factor < self.star.minimum_fill_factor {
            let mut generator = self
                .generator
                .write()
                .expect("Simulation RNG lock was poisoned by another panic");

            while current_fill_factor < self.star.minimum_fill_factor {
                let new_fill_factor = iter::repeat(())
                    .map(|_| fill_range.ind_sample(&mut *generator) * 9.4e-6)
                    .find(|v| *v < 0.001)
                    .unwrap();

                let mut new_spot = Spot::from_config(
                    self.star.clone(),
                    &SpotConfig {
                        latitude: lat_range.ind_sample(&mut *generator),
                        longitude: long_range.ind_sample(&mut *generator),
                        fill_factor: new_fill_factor,
                        plage: false,
                    },
                );
                new_spot.mortality = Mortal(Bounds::new(time, time + 15.0));

                // TODO: This is a hack
                let new_appear = time;
                let new_disappear = time + 15.0;
                let collides = self
                    .spots
                    .iter()
                    .filter(|s| s.alive(new_appear) || s.alive(new_disappear))
                    .any(|s| new_spot.collides_with(s));

                if !collides {
                    current_fill_factor += (new_spot.radius * new_spot.radius) / 2.0;
                    self.spots.push(new_spot);
                }
            }
        }
    }

    /// Computes the relative brightness of this system at each time (in days),
    /// when observed in the wavelength band between `wavelength_min` and `wavelength_max`.
    pub fn observe_flux(&mut self, time: &[f64], wavelength: Bounds) -> Vec<f64> {
        let star_intensity =
            planck_integral(self.star.temperature, wavelength.lower, wavelength.upper);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength.lower, wavelength.upper)
                / star_intensity;
        }
        for t in time.iter() {
            self.check_fill_factor(*t);
        }

        time.par_iter()
            .map(|t| {
                let spot_flux: f64 = self.spots.iter().map(|s| s.get_flux(*t)).sum();
                (self.star.flux_quiet - spot_flux) / self.star.flux_quiet
            })
            .collect()
    }

    /// Computes the radial velocity and line bisector of this system at each time (in days),
    /// when observed in the wavelength band between `wavelength_min` and `wavelength_max`.
    pub fn observe_rv(&mut self, time: &[f64], wavelength: Bounds) -> Vec<Observation> {
        let star_intensity =
            planck_integral(self.star.temperature, wavelength.lower, wavelength.upper);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength.lower, wavelength.upper)
                / star_intensity;
        }

        for t in time.iter() {
            self.check_fill_factor(*t);
        }

        time.par_iter()
            .map(|t| {
                let mut spot_profile = vec![0.0; self.star.profile_spot.len()];
                for spot in self.spots.iter().filter(|s| s.alive(*t)) {
                    let profile = spot.get_ccf(*t);
                    for (total, this) in spot_profile.iter_mut().zip(profile.iter()) {
                        *total += *this;
                    }
                }

                for (spot, star) in spot_profile.iter_mut().zip(self.star.integrated_ccf.iter()) {
                    *spot = *star - *spot;
                }

                /*
                use resolution::set_resolution;
                let spot_profile = set_resolution(&self.star.profile_spot.rv, &spot_profile);
                println!("{:?}", spot_profile);
                panic!();
                */

                let rv = fit_rv(&self.star.profile_quiet.rv, &spot_profile) - self.star.zero_rv;

                let bisector: Vec<f64> = compute_bisector(&self.star.profile_quiet.rv, &spot_profile)
                        .iter()
                        // TODO: Should I actually return the points that come back from this?
                        // Do the Y values actually matter?
                        //.map(|b| b.x - self.star.zero_rv)
                        .map(|b| b - self.star.zero_rv)
                        .collect();

                Observation { rv, bisector }
            })
            .collect()
    }

    /// Draw the simulation in a row-major fashion, as it would be seen in the visible
    /// wavelength band, 4000-7000 Angstroms.
    pub fn draw_rgba(&mut self, time: f64) -> Vec<u8> {
        // This is slow because the image is row-major, but we navigate the simulation in
        // a column-major fashion to follow the rotational symmetry
        use boundingshape::BoundingShape;
        use linspace::floatrange;
        let mut image = self.star.draw_rgba();

        self.check_fill_factor(time);
        let star_intensity = planck_integral(self.star.temperature, 4000e-10, 7000e-10);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, 4000e-10, 7000e-10) / star_intensity;
        }

        let grid_interval = 2.0 / self.star.grid_size as f64;

        for spot in self.spots.iter().filter(|s| s.alive(time)) {
            let bounds = BoundingShape::new(spot, time);
            let mut current_z_bounds = None;
            if let Some(y_bounds) = bounds.y_bounds() {
                for y in floatrange(
                    (y_bounds.lower / grid_interval).round() * grid_interval,
                    (y_bounds.upper / grid_interval).round() * grid_interval,
                    grid_interval,
                ) {
                    let y_index = ((y + 1.0) / 2.0 * 1000.0).round() as usize;
                    if let Some(z_bounds) = bounds.z_bounds(y, &mut current_z_bounds) {
                        for z in floatrange(
                            (z_bounds.lower / grid_interval).round() * grid_interval,
                            (z_bounds.upper / grid_interval).round() * grid_interval,
                            grid_interval,
                        ) {
                            let x = 1.0 - (y * y + z * z);
                            let x = f64::max(0.0, x);
                            let intensity = self.star.limb_brightness(x) * spot.intensity;
                            let z_index = ((-z + 1.0) / 2.0 * 1000.0).round() as usize;
                            let index = (z_index * 1000 + y_index) as usize;
                            image[4 * index] = (intensity * 255.0) as u8;
                            image[4 * index + 1] = (intensity * 131.0) as u8;
                            image[4 * index + 2] = 0;
                            image[4 * index + 3] = 255;
                        }
                    }
                }
            }
        }
        image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_is_valid() {
        Simulation::from_config("../examples/sun.toml");
    }
}
