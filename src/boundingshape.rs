extern crate std;
use std::f64::consts;

use point::Point;
use spot::Spot;

pub struct Bounds {
    pub lower: f64,
    pub upper: f64
}

impl Bounds {
    pub fn new(val1: f64, val2: f64) -> Self {
        if val1 <= val2 {
            return Bounds {
                lower: val1,
                upper: val2,
            };
        } else {
            return Bounds {
                lower: val2,
                upper: val1,
            };
        }
    }
}

pub struct BoundingShape {
    center: Point,
    circle_center: Point,
    a: Point,
    b: Point,
    radius: f64,
    grid_interval: f64,
    max_radius: f64,
    visible: bool,
    is_on_edge: bool,
}

impl BoundingShape {
    pub fn new(spot: &Spot, time: f64) -> Self {
        // TODO: Fix these hacks by implementing Star properly
        let period = 25.05;
        let grid_interval = 0.01;
        let inclination = 0.0;

        let mut radius = spot.radius;
        let max_radius = radius;

        if spot.mortal {
            let lifetime = spot.time_disappear - spot.time_appear;
            let growth_time = 0.1*lifetime;
            if (time - spot.time_appear).abs() < growth_time {
                radius *= (time - spot.time_appear).abs() / growth_time;
            }
            else if (time - spot.time_disappear).abs() < growth_time {
                radius *= (time - spot.time_disappear).abs() / growth_time;
            }
        }

        let phase = (time % period) / period * 2.0 * consts::PI;
        let theta = phase + spot.longitude;
        let phi = consts::FRAC_PI_2 - spot.longitude;
        let mut center = Point {
            x: phi.sin() * theta.cos(),
            y: phi.sin() * theta.sin(),
            z: phi.cos(),
        };
        center.rotate_y(inclination - consts::FRAC_PI_2);

        let depth = (1.0 - radius*radius).sqrt();
        let circle_center = Point {
            x: center.x * depth,
            y: center.y * depth,
            z: center.z * depth,
        };

        let a_y = -circle_center.z/(circle_center.y*circle_center.y + circle_center.z*circle_center.z).sqrt();
        let a = Point {
            x: 0.0,
            y: a_y,
            z: (1.0 - a_y * a_y).sqrt(),
        };

        let b = Point {
            x: circle_center.y*a.z - circle_center.z*a.y,
            y: circle_center.z*a.x - circle_center.x*a.z,
            z: circle_center.x*a.y - circle_center.y*a.x,
        };

        let theta_x_max = -2.0 * ((a.x - (a.x * a.x + b.x * b.x).sqrt()) / b.x).atan();
        let theta_x_min = -2.0 * ((a.x + (a.x * a.x + b.x * b.x).sqrt()) / b.x).atan();

        let x1 = circle_center.x + radius*((theta_x_max).cos()*a.x + (theta_x_max).sin()*b.x);
        let x2 = circle_center.x + radius*((theta_x_min).cos()*a.x + (theta_x_min).sin()*b.x);

        let is_on_edge = x1 < 0.0 || x2 < 0.0;
        let visible = x1 > 0.0 || x2 > 0.0;

        BoundingShape {
            center: center,
            circle_center: circle_center,
            a: a,
            b: b,
            radius: radius,
            grid_interval: grid_interval,
            max_radius: max_radius,
            visible: visible,
            is_on_edge: is_on_edge,
        }
    }

    pub fn y_bounds(&self) -> Bounds {
        if !self.visible {
            return Bounds::new(0.0, 0.0);
        }
        let mut theta_y_max = consts::PI;
        let mut theta_y_min = 0.0;
        if self.b.y != 0.0 {
            theta_y_max = -2.0 * ((self.a.y - (self.a.y * self.a.y + self.b.y * self.b.y).sqrt()) / self.b.y).atan();
            theta_y_min = -2.0 * ((self.a.y + (self.a.y * self.a.y + self.b.y * self.b.y).sqrt()) / self.b.y).atan();
        }

        let y_max = self.circle_center.y + self.radius*(theta_y_max.cos()*self.a.y + theta_y_max.sin()*self.b.y);
        let y_min = self.circle_center.y + self.radius*(theta_y_min.cos()*self.a.y + theta_y_min.sin()*self.b.y);

        // TODO: FIX THIS
        let x_max = self.circle_center.x + self.radius*(theta_y_max.cos()*self.a.y + theta_y_max.sin()*self.b.y);
        let x_min = self.circle_center.x + self.radius*(theta_y_max.cos()*self.b.y + theta_y_max.sin()*self.b.y);

        if x_min < 0.0 && x_max < 0.0 {
            return Bounds::new(0.0, 0.0);
        }
        if x_max < 0.0 {
            return Bounds::new(y_min, 0.0);
        }
        if x_min < 0.0 {
            return Bounds::new(y_max, -1.0);
        }
        Bounds::new(y_min,  y_max)
    }

    pub fn z_bounds(&self, y: f64) -> Bounds {
        if self.radius == 0.0 {
            return Bounds::new(0.0, 0.0);
        }

        let mut z_max = 2.0;
        let mut z_min = 2.0;
        let mut z = 0.0;

        z = self.center.z+self.radius;
        while (z > self.center.z - self.radius) && self.on_spot(y, z) {
            z_max = z;
            z -= self.grid_interval;
        }

        z = self.center.z - self.radius;
        while (z < self.center.z - self.radius) && self.on_spot(y, z) {
            z_min = z;
            z += self.grid_interval;
        }

        Bounds::new(z_min, z_max)
    }

    pub fn on_spot(&self, y: f64, z: f64) -> bool {
        if !on_star(y, z) {
            return false
        }
        let x = (1.0 - (y*y + z*z)).sqrt();
        let distance_squared = (y-self.circle_center.y)*(y-self.circle_center.y) +
            (z-self.circle_center.z)*(z-self.circle_center.z) +
            (x-self.circle_center.x)*(x-self.circle_center.x);

        distance_squared <= (self.radius * self.radius)
    }

    pub fn collides_with(&self, other: &BoundingShape) -> bool {
        let distance = (
            (self.center.x - other.center.x).powi(2) +
            (self.center.y - other.center.y).powi(2) +
            (self.center.z - other.center.z).powi(2)
            ).sqrt();
        distance < (self.max_radius + other.max_radius)
    }
}

fn on_star(y: f64, z: f64) -> bool {
    (y*y + z*z) <= 1.0
}
