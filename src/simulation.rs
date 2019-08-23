use std::path::Path;
use std::sync::{Arc, Mutex};

use rand::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::boundingshape::BoundingShape;
use crate::bounds::Bounds;
use crate::linspace::floatrange;
use crate::planck::planck_integral;
use crate::solar_ccfs::CCF_LEN;
use crate::spot::Mortality::Mortal;
use crate::spot::{Spot, SpotConfig};
use crate::star::{Star, StarConfig};

/// A model of a star with spots that can be observed.
pub struct Simulation {
    pub star: Arc<Star>,
    pub spots: Vec<Spot>,
    generator: Arc<Mutex<StdRng>>,
}

impl std::fmt::Debug for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Simulation")
            .field("star", &self.star)
            .field("spots", &self.spots)
            .finish()
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum SeedConfig {
    Number(u64),
    Text(String),
}

#[derive(Deserialize, Serialize)]
struct Config {
    seed: Option<SeedConfig>,
    star: StarConfig,
    spots: Option<Vec<SpotConfig>>,
}

impl Config {
    // This is only used to produce our pretty example if somebody feeds in a bad config
    fn example() -> Config {
        Config {
            seed: None,
            star: StarConfig {
                grid_size: 1000,
                radius: 1.0,
                period: 25.05,
                inclination: 90.0,
                temperature: 5778.0,
                spot_temp_diff: 663.0,
                limb_linear: 0.29,
                limb_quadratic: 0.34,
                target_fill_factor: Some(0.00),
                minimum_fill_factor: None,
                latitude_distribution: None,
                longitude_distribution: None,
                fillfactor_distribution: None,
                lifetime_distribution: None,
            },
            spots: Some(vec![
                SpotConfig {
                    latitude: 30.0,
                    longitude: 180.0,
                    fill_factor: 0.01,
                    plage: false,
                    temperature: None,
                },
                SpotConfig {
                    latitude: 30.0,
                    longitude: 180.0,
                    fill_factor: 0.01,
                    plage: false,
                    temperature: None,
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
                target_fill_factor: Some(0.01),
                minimum_fill_factor: None,
                latitude_distribution: None,
                longitude_distribution: None,
                fillfactor_distribution: None,
                lifetime_distribution: None,
            })),
            spots: Vec::new(),
            generator: Arc::new(Mutex::new(StdRng::seed_from_u64(0x0123456789ABCDEFu64))),
        }
    }

    /// Construct a new Star from a TOML file.
    pub fn from_config(config_path: &Path) -> Result<Simulation, String> {
        let contents = std::fs::read_to_string(config_path).map_err(|_| {
            format!(
                "Tried to open a config file at {:?}, but it doesn't seem to exist",
                config_path
            )
        })?;

        let config: Config = ::toml::from_str(&contents).map_err(|e| {
            format!(
                "{:?} is not a valid config file. Here's an example:\n{}\nThe internal error was: {}",
                &config_path,
                ::toml::to_string_pretty(&Config::example()).unwrap(),
                e
            )
        })?;

        let rng = match config.seed {
            Some(SeedConfig::Number(num)) => StdRng::seed_from_u64(num),
            Some(SeedConfig::Text(t)) => {
                if &t == "entropy" {
                    eprintln!("Seeding simulation RNG from system-provided entropy");
                    StdRng::from_entropy()
                } else {
                    return Err(
                    "Invalid rng seed specification, valid seeds are \"entropy\", or an integer"
                        .to_string());
                }
            }
            None => StdRng::seed_from_u64(0x0123456789ABCDEFu64),
        };

        let mut sim = Simulation {
            star: Arc::new(Star::from_config(&config.star)),
            spots: Vec::new(),
            generator: Arc::new(Mutex::new(rng)),
        };

        if let Some(spot_configs) = config.spots {
            for spot_config in spot_configs {
                sim.spots
                    .push(Spot::from_config(Arc::clone(&sim.star), &spot_config));
            }
        }

        Ok(sim)
    }

    pub fn add_spot(&mut self, config: &SpotConfig) {
        self.spots
            .push(Spot::from_config(Arc::clone(&self.star), config));
    }

    pub fn clear_spots(&mut self) {
        self.spots.clear();
    }

    pub fn check_fill_factor(&mut self, time: f64) {
        let mut current_fill_factor = self
            .spots
            .iter()
            .filter(|s| s.alive(time))
            .map(|s| (s.radius * s.radius) / 2.0)
            .sum::<f64>();

        let mut generator = self
            .generator
            .lock()
            .expect("Simulation RNG lock was poisoned by another panic");

        while current_fill_factor < self.star.target_fill_factor {
            let new_fill_factor = loop {
                let possible_value =
                    self.star.fillfactor_distribution.sample(&mut *generator) * 9.4e-6;
                if possible_value < 0.001 {
                    break possible_value;
                }
            };

            let mut new_spot = Spot::from_config(
                self.star.clone(),
                &SpotConfig {
                    latitude: self.star.latitude_distribution.sample(&mut *generator),
                    longitude: self.star.longitude_distribution.sample(&mut *generator),
                    fill_factor: new_fill_factor,
                    plage: false,
                    temperature: None,
                },
            );
            new_spot.mortality = Mortal(Bounds::new(
                time,
                time + self.star.lifetime_distribution.sample(&mut *generator),
            ));

            // TODO: This collision checking might be subpar
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

    /// Computes the relative brightness of this system at each time (in days),
    /// when observed in the wavelength band between `wavelength_min` and `wavelength_max`.
    pub fn observe_flux(&mut self, time: &[f64], wavelength: Bounds) -> Vec<f64> {
        for t in time.iter() {
            self.check_fill_factor(*t);
        }

        let star_intensity =
            planck_integral(self.star.temperature, wavelength.lower, wavelength.upper);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength.lower, wavelength.upper)
                / star_intensity;
        }

        time.par_iter()
            .map(|t| {
                let spot_flux: f64 = self
                    .spots
                    .iter()
                    .filter(|s| s.alive(*t))
                    .map(|s| s.get_flux(*t))
                    .sum();
                (self.star.flux_quiet - spot_flux) / self.star.flux_quiet
            })
            .collect()
    }

    /// Computes the radial velocity and line bisector of this system at each time (in days),
    /// when observed in the wavelength band between `wavelength_min` and `wavelength_max`.
    pub fn observe_rv(&mut self, time: &[f64], wavelength: Bounds) -> Vec<Vec<f64>> {
        for t in time.iter() {
            self.check_fill_factor(*t);
        }

        let star_intensity =
            planck_integral(self.star.temperature, wavelength.lower, wavelength.upper);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength.lower, wavelength.upper)
                / star_intensity;
        }

        time.par_iter()
            .map(|t| {
                let mut spots_profile = vec![0.0; CCF_LEN];
                for spot in self.spots.iter().filter(|s| s.alive(*t)) {
                    let this_profile = spot.get_ccf(*t);
                    for (total, this) in spots_profile.iter_mut().zip(this_profile.iter()) {
                        *total += *this;
                    }
                }

                for (spot, star) in spots_profile
                    .iter_mut()
                    .zip(self.star.integrated_ccf.iter())
                {
                    *spot = *star - *spot;
                }

                spots_profile
            })
            .collect()
    }

    /// Draw the simulation in a row-major fashion, as it would be seen in the visible
    /// wavelength band, 4000-7000 Angstroms.
    pub fn draw_bgr(&mut self, time: f64, image: &mut [u8]) {
        // This is slow because the image is row-major, but we navigate the simulation in
        // a column-major fashion to follow the rotational symmetry
        self.check_fill_factor(time);

        self.star.draw_bgr(image);

        let star_intensity = planck_integral(self.star.temperature, 4000e-10, 7000e-10);
        // Clone the spots and mutate a local copy so this function runs in parallel
        let mut spots = self.spots.clone();
        for spot in &mut spots {
            spot.intensity = planck_integral(spot.temperature, 4000e-10, 7000e-10) / star_intensity;
        }

        let grid_interval = 2.0 / self.star.grid_size as f64;

        for spot in spots.iter().filter(|s| s.alive(time)) {
            let color = match &TEMP_TO_RGB.binary_search_by(|k| k.0.cmp(&(spot.temperature as u16)))
            {
                Ok(v) => TEMP_TO_RGB[*v].1,
                Err(v) => TEMP_TO_RGB[*v].1,
            };
            let color = [color[0] as f64, color[1] as f64, color[2] as f64];

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
                            // opencv wants BGR, we have RGB
                            image[3 * index + 0] = min(color[2] * intensity, 255.0) as u8;
                            image[3 * index + 1] = min(color[1] * intensity, 255.0) as u8;
                            image[3 * index + 2] = min(color[0] * intensity, 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
}

fn min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

//http://www.isthe.com/chongo/tech/astro/HR-temp-mass-table-byhrclass.html

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_is_valid() {
        Simulation::from_config(&Path::new("../examples/sun.toml")).unwrap();
        Simulation::from_config(&Path::new("../examples/random.toml")).unwrap();
    }
}

static TEMP_TO_RGB: [(u16, [u8; 3]); 130] = [
    (1990, [255, 233, 154]),
    (2000, [255, 198, 108]),
    (2167, [255, 198, 109]),
    (2180, [255, 167, 97]),
    (2333, [255, 197, 111]),
    (2370, [255, 165, 97]),
    (2500, [255, 195, 112]),
    (2560, [255, 178, 121]),
    (2667, [255, 204, 111]),
    (2750, [255, 197, 124]),
    (2833, [255, 201, 127]),
    (2940, [255, 206, 127]),
    (3000, [255, 206, 129]),
    (3130, [255, 200, 119]),
    (3167, [255, 196, 131]),
    (3320, [255, 198, 118]),
    (3333, [255, 204, 142]),
    (3500, [255, 195, 139]),
    (3510, [255, 200, 121]),
    (3640, [255, 202, 157]),
    (3700, [255, 203, 132]),
    (3717, [255, 206, 140]),
    (3725, [255, 205, 135]),
    (3733, [255, 208, 142]),
    (3750, [255, 206, 139]),
    (3780, [255, 209, 174]),
    (3800, [255, 211, 146]),
    (3900, [255, 216, 167]),
    (3920, [255, 199, 142]),
    (4000, [255, 221, 175]),
    (4057, [255, 223, 181]),
    (4060, [255, 205, 152]),
    (4114, [255, 227, 190]),
    (4171, [255, 231, 196]),
    (4200, [255, 210, 161]),
    (4229, [255, 231, 199]),
    (4286, [255, 234, 207]),
    (4340, [255, 216, 181]),
    (4343, [255, 236, 215]),
    (4400, [255, 236, 211]),
    (4480, [255, 222, 195]),
    (4620, [255, 227, 196]),
    (4669, [255, 243, 233]),
    (4760, [255, 224, 188]),
    (4900, [255, 238, 221]),
    (4937, [255, 243, 233]),
    (5010, [255, 239, 221]),
    (5120, [255, 237, 222]),
    (5206, [255, 243, 233]),
    (5230, [255, 244, 235]),
    (5340, [255, 244, 235]),
    (5450, [255, 244, 234]),
    (5474, [255, 243, 233]),
    (5560, [255, 241, 229]),
    (5670, [255, 243, 236]),
    (5743, [255, 242, 233]),
    (5780, [255, 245, 242]),
    (5890, [255, 247, 248]),
    (6000, [255, 248, 252]),
    (6011, [255, 246, 233]),
    (6140, [255, 247, 252]),
    (6280, [255, 247, 252]),
    (6420, [246, 243, 255]),
    (6520, [255, 243, 250]),
    (6560, [244, 241, 255]),
    (6700, [248, 247, 255]),
    (6760, [255, 234, 252]),
    (6840, [224, 226, 255]),
    (6980, [230, 233, 255]),
    (7000, [219, 225, 255]),
    (7120, [236, 239, 255]),
    (7193, [227, 231, 255]),
    (7260, [230, 234, 255]),
    (7387, [236, 237, 255]),
    (7400, [224, 229, 255]),
    (7580, [244, 243, 255]),
    (7650, [219, 224, 255]),
    (7773, [223, 229, 255]),
    (7900, [213, 222, 255]),
    (7967, [202, 215, 255]),
    (8150, [200, 213, 255]),
    (8160, [206, 218, 255]),
    (8353, [210, 221, 255]),
    (8400, [199, 212, 255]),
    (8547, [215, 223, 255]),
    (8650, [202, 215, 255]),
    (8740, [219, 226, 255]),
    (8900, [197, 211, 255]),
    (8933, [223, 229, 255]),
    (9127, [215, 224, 255]),
    (9150, [191, 207, 255]),
    (9320, [207, 219, 255]),
    (9400, [187, 203, 255]),
    (9513, [199, 214, 255]),
    (9650, [181, 199, 255]),
    (9707, [214, 223, 255]),
    (9900, [185, 201, 255]),
    (11710, [181, 198, 255]),
    (13520, [177, 195, 255]),
    (15330, [173, 191, 255]),
    (17140, [172, 189, 255]),
    (18950, [170, 191, 255]),
    (20160, [187, 203, 255]),
    (20760, [164, 184, 255]),
    (21370, [175, 194, 255]),
    (22570, [165, 185, 255]),
    (22580, [177, 196, 255]),
    (23790, [168, 193, 255]),
    (24380, [160, 180, 255]),
    (25000, [161, 189, 255]),
    (26190, [160, 182, 255]),
    (27600, [164, 185, 255]),
    (28000, [156, 178, 255]),
    (30200, [154, 178, 255]),
    (32400, [157, 177, 255]),
    (32800, [160, 181, 255]),
    (34600, [157, 177, 255]),
    (35400, [157, 178, 255]),
    (36800, [162, 184, 255]),
    (38000, [155, 176, 255]),
    (39000, [155, 176, 255]),
    (40400, [153, 174, 255]),
    (41200, [153, 174, 255]),
    (42800, [151, 172, 255]),
    (43400, [151, 172, 255]),
    (45200, [148, 170, 255]),
    (45600, [148, 170, 255]),
    (47600, [146, 168, 255]),
    (47800, [146, 168, 255]),
    (50000, [144, 166, 255]),
];
