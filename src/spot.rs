extern crate std;
use std::f64::consts;

use star::Star;
use boundingshape::BoundingShape;
use float_range::FloatRange;

pub struct Spot {
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub temperature: f64,
    pub plage: bool,
    pub mortal: bool,
    pub time_appear: f64,
    pub time_disappear: f64,
}

impl Spot {
    pub fn new(latitude: f64, longitude: f64, fillfactor: f64, plage: bool, mortal: bool) -> Self {
        let this = Spot {
            latitude: latitude * consts::PI / 180.0,
            longitude: longitude * consts::PI / 180.0,
            radius: (2.0 * fillfactor).sqrt(),
            temperature: 0.0,
            plage: plage,
            mortal: mortal,
            time_appear: 0.0,
            time_disappear: 15.0,
        };
        if plage {
            panic!("Plages are not implemented");
        }
        return this;
    }

    pub fn get_flux(&self, time: f64) {

        let grid_interval = 0.01;

        let mut limb_integral = 0.0;
        let bounds = BoundingShape::new(&self, time);
        let y_bounds = bounds.y_bounds();
        let mut y = y_bounds.lower;
        for y in FloatRange(y_bounds.lower, y_bounds.upper, grid_interval) {

        }
    }
}
