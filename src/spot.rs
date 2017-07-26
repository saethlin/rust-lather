extern crate std;
extern crate itertools;
use std::f64::consts;
use std::sync::Arc;
use self::itertools::cons_tuples;

use star::Star;
use boundingshape::BoundingShape;
use linspace::floatrange;

pub struct Spot {
    pub star: Arc<Star>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub temperature: f64,
    pub plage: bool,
    pub mortal: bool,
    pub time_appear: f64,
    pub time_disappear: f64,
    pub intensity: f64,
}

impl Spot {
    pub fn new(star: Arc<Star>, latitude: f64, longitude: f64, fillfactor: f64, plage: bool, mortal: bool)
        -> Self {
        let temperature = star.temperature - star.spot_temp_diff;
        Spot {
            star: star,
            latitude: latitude * consts::PI / 180.0,
            longitude: longitude * consts::PI / 180.0,
            radius: (2.0 * fillfactor).sqrt(),
            temperature: temperature,
            plage: plage,
            mortal: mortal,
            time_appear: 0.0,
            time_disappear: 15.0,
            intensity: 0.0,
        }
    }

    pub fn get_flux(&self, time: f64) -> f64 {
        let bounds = BoundingShape::new(self, time);
        if let Some(y_bounds) = bounds.y_bounds() {
            let limb_integral: f64 = floatrange(
                y_bounds.lower,
                y_bounds.upper,
                2.0 / self.star.grid_size as f64,
            ).map(|y| { // TODO I should be able to filter_map here
                if let Some(z_bounds) = bounds.z_bounds(y) {
                    self.star.limb_integral(&z_bounds, y)
                } else {
                    0.0
                }
            })
                .sum();
            (1.0 - self.intensity) * limb_integral
        } else {
            0.0
        }
    }

    pub fn get_ccf(&self, time: f64) -> Vec<f64> {
        let mut profile = vec![0.0; self.star.profile_active.len()];
        let bounds = BoundingShape::new(self, time);
        if let Some(y_bounds) = bounds.y_bounds() {
            for y in floatrange(
                y_bounds.lower,
                y_bounds.upper,
                2.0 / self.star.grid_size as f64,
            )
            {
                let quiet_shifted = self.star.profile_quiet.shift(
                    y * self.star.equatorial_velocity,
                );
                let active_shifted = self.star.profile_active.shift(
                    y * self.star.equatorial_velocity,
                );

                if let Some(z_bounds) = bounds.z_bounds(y) {
                    let limb_integral = self.star.limb_integral(&z_bounds, y);
                    for (tot, qshift, ashift) in
                        cons_tuples(profile.iter_mut().zip(quiet_shifted.iter()).zip(
                            active_shifted.iter(),
                        ))
                    {
                        *tot += (qshift - self.intensity * ashift) * limb_integral;
                    }
                }
            }
        }; // TODO: I'm not clear on why I need a semicolon here
        profile
    }

    pub fn alive(&self, time: f64) -> bool {
        if !self.mortal {
            true
        } else {
            time >= self.time_appear && time <= self.time_disappear
        }
    }

    pub fn collides_with(&self, other: &Spot) -> bool {
        let bounds = BoundingShape::new(self, 0.0);
        let other_bounds = BoundingShape::new(other, 0.0);
        bounds.collides_with(&other_bounds)
    }
}

impl std::fmt::Debug for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("")
            .field("latitude", &self.latitude)
            .field("longitude", &self.longitude)
            .field("radius", &self.radius)
            .field("plage", &self.plage)
            .field("mortal", &self.mortal)
            .finish()
    }
}
