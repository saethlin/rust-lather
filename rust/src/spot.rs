use std::f64::consts;
use std::sync::Arc;

use boundingshape::BoundingShape;
use bounds::Bounds;
use linspace::floatrange;
use star::Star;

#[derive(Deserialize, Serialize)]
pub struct SpotConfig {
    pub latitude: f64,
    pub longitude: f64,
    pub fill_factor: f64,
    pub plage: bool,
}

/// A circular starspot
pub struct Spot {
    pub star: Arc<Star>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub temperature: f64,
    pub plage: bool,
    pub mortality: Mortality,
    pub intensity: f64,
}

impl std::fmt::Debug for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Spot")
            .field("latitidue", &self.latitude)
            .field("longitude", &self.longitude)
            .field("radius", &self.radius)
            .field("temperature", &self.temperature)
            .field("plage", &self.plage)
            .field("mortality", &self.mortality)
            .field("intensity", &self.intensity)
            .finish()
    }
}

#[derive(Debug)]
pub enum Mortality {
    Immortal,
    Mortal(Bounds),
}

impl Default for Mortality {
    fn default() -> Self {
        Mortality::Immortal
    }
}

impl Spot {
    /// Create a new spot on `star` and at the coordinates specified, where
    /// latitude is 0 at the equator and both latitude and longitude are
    /// measured in degrees.
    pub fn from_config(star: Arc<Star>, config: &SpotConfig) -> Spot {
        let temperature = star.temperature - star.spot_temp_diff;
        Spot {
            star,
            latitude: config.latitude * consts::PI / 180.0,
            longitude: config.longitude * consts::PI / 180.0,
            radius: (2.0 * config.fill_factor).sqrt(),
            temperature,
            plage: config.plage,
            mortality: Mortality::Immortal,
            intensity: 0.0,
        }
    }

    pub fn get_flux(&self, time: f64) -> f64 {
        let bounds = BoundingShape::new(self, time);
        if let Some(y_bounds) = bounds.y_bounds() {
            let mut current_z_bounds = None;

            let limb_integral: f64 = floatrange(
                y_bounds.lower,
                y_bounds.upper,
                2.0 / self.star.grid_size as f64,
            ).map(|y| {
                // TODO I should be able to filter_map here
                if let Some(z_bounds) = bounds.z_bounds(y, &mut current_z_bounds) {
                    self.star.limb_integral(&z_bounds, y)
                } else {
                    0.0
                }
            }).sum();
            (1.0 - self.intensity) * limb_integral
        } else {
            0.0
        }
    }

    pub fn get_ccf(&self, time: f64) -> Vec<f64> {
        let mut profile = vec![0.0; self.star.profile_spot.len()];
        let mut quiet_shifted = vec![0.0; self.star.profile_quiet.len()];
        let mut active_shifted = vec![0.0; self.star.profile_spot.len()];
        let bounds = BoundingShape::new(self, time);
        let mut current_z_bounds = None;
        if let Some(y_bounds) = bounds.y_bounds() {
            for y in floatrange(
                y_bounds.lower,
                y_bounds.upper,
                2.0 / self.star.grid_size as f64,
            ) {
                self.star
                    .profile_quiet
                    .shift_into(y * self.star.equatorial_velocity, &mut quiet_shifted);
                self.star
                    .profile_spot
                    .shift_into(y * self.star.equatorial_velocity, &mut active_shifted);

                if let Some(z_bounds) = bounds.z_bounds(y, &mut current_z_bounds) {
                    let limb_integral = self.star.limb_integral(&z_bounds, y);
                    for ((tot, qshift), ashift) in profile
                        .iter_mut()
                        .zip(quiet_shifted.iter())
                        .zip(active_shifted.iter())
                    {
                        *tot += (qshift - self.intensity * ashift) * limb_integral;
                    }
                }
            }
        };
        profile
    }

    /// Returns `true` if this spot currently exists.
    pub fn alive(&self, time: f64) -> bool {
        match self.mortality {
            Mortality::Immortal => true,
            Mortality::Mortal(lifetime) => time >= lifetime.lower && time <= lifetime.upper,
        }
    }

    /// Returns whether a spot _ever_ collides with the `other`.
    pub fn collides_with(&self, other: &Spot) -> bool {
        let bounds = BoundingShape::new(self, 0.0);
        let other_bounds = BoundingShape::new(other, 0.0);
        bounds.collides_with(&other_bounds)
    }
}
