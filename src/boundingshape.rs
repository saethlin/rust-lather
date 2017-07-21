extern crate std;
use std::f64::consts;

use point::Point;
use spot::Spot;
use bounds::Bounds;

pub struct BoundingShape {
    center: Point,
    circle_center: Point,
    a: Point,
    b: Point,
    radius: f64,
    max_radius: f64,
    visible: bool,
}

impl BoundingShape {
    pub fn new(spot: &Spot, time: f64) -> Self {
        let max_radius = spot.radius;
        let radius = if spot.mortal {
            let lifetime = spot.time_disappear - spot.time_appear;
            let growth_time = 0.1 * lifetime;
            if (time - spot.time_appear).abs() < growth_time {
                spot.radius *  (time - spot.time_appear).abs() / growth_time
            } else if (time - spot.time_disappear).abs() < growth_time {
                spot.radius *  (time - spot.time_disappear).abs() / growth_time
            } else {
                spot.radius
            }
        } else {
            spot.radius
        };

        let phase = (time % spot.star.period) / spot.star.period * 2.0 * consts::PI;
        let theta = phase + spot.longitude;
        let phi = consts::FRAC_PI_2 - spot.latitude;

        let center = Point {
            x: phi.sin() * theta.cos(),
            y: phi.sin() * theta.sin(),
            z: phi.cos(),
        }.rotated_y(spot.star.inclination - consts::FRAC_PI_2);

        let depth = (1.0 - radius.powi(2)).sqrt();
        let circle_center = Point {
            x: center.x * depth,
            y: center.y * depth,
            z: center.z * depth,
        };

        let a_y = -circle_center.z / (circle_center.y.powi(2) + circle_center.z.powi(2)).sqrt();
        let a = Point {
            x: 0.0,
            y: a_y,
            z: (1.0 - a_y.powi(2)).sqrt(),
        };

        let b = Point {
            x: circle_center.y * a.z - circle_center.z * a.y,
            y: circle_center.z * a.x - circle_center.x * a.z,
            z: circle_center.x * a.y - circle_center.y * a.x,
        };

        let theta_x_max = -2.0 * ((a.x - (a.x.powi(2) + b.x.powi(2)).sqrt()) / b.x).atan();
        let theta_x_min = -2.0 * ((a.x + (a.x.powi(2) + b.x.powi(2)).sqrt()) / b.x).atan();

        let x1 = circle_center.x + radius * ((theta_x_max).cos() * a.x + (theta_x_max).sin() * b.x);
        let x2 = circle_center.x + radius * ((theta_x_min).cos() * a.x + (theta_x_min).sin() * b.x);

        let visible = x1 > 0.0 || x2 > 0.0;

        BoundingShape {
            center: center,
            circle_center: circle_center,
            a: a,
            b: b,
            radius: radius,
            max_radius: max_radius,
            visible: visible,
        }
    }

    pub fn y_bounds(&self) -> Bounds {
        if !self.visible {
            return Bounds::new(0.0, 0.0);
        }

        let theta_y_min = if self.b.y != 0.0 {
            -2.0 * ((self.a.y + (self.a.y.powi(2) + self.b.y.powi(2)).sqrt()) / self.b.y).atan()
        } else {
            0.0
        };
        let theta_y_max = if self.b.y != 0.0 {
            -2.0 * ((self.a.y - (self.a.y.powi(2) + self.b.y.powi(2)).sqrt()) / self.b.y).atan()
        } else {
            consts::PI
        };

        let y_max = self.circle_center.y +
            self.radius * (theta_y_max.cos() * self.a.y + theta_y_max.sin() * self.b.y);
        let y_min = self.circle_center.y +
            self.radius * (theta_y_min.cos() * self.a.y + theta_y_min.sin() * self.b.y);

        let x_max = self.circle_center.x +
            self.radius * (theta_y_max.cos() * self.a.y + theta_y_max.sin() * self.b.y);
        let x_min = self.circle_center.x +
            self.radius * (theta_y_max.cos() * self.b.y + theta_y_max.sin() * self.b.y);

        if x_min < 0.0 && x_max < 0.0 {
            return Bounds::new(0.0, 0.0);
        }
        if x_max < 0.0 {
            return Bounds::new(y_min, 0.0);
        }
        if x_min < 0.0 {
            return Bounds::new(y_max, -1.0);
        }

        Bounds::new(y_min, y_max)
    }

    pub fn z_bounds(&self, y: f64) -> Bounds {
        if self.radius == 0.0 {
            return Bounds::new(0.0, 0.0);
        }
        let y_mod = (y - self.circle_center.y) / self.radius;
        let tmp = (self.a.y.powi(2) + self.b.y.powi(2) - y_mod.powi(2)).sqrt();
        if tmp.is_nan() {
            return Bounds::new(0.0, 0.0);
        }
        let theta1 = 2.0 * ((self.b.y + tmp) / (self.a.y + y_mod)).atan();
        let theta2 = 2.0 * ((self.b.y - tmp) / (self.a.y + y_mod)).atan();

        let z1: f64 = self.circle_center.z +
            self.radius * (self.a.z * theta1.cos() + self.b.z * theta1.sin());
        let z2: f64 = self.circle_center.z +
            self.radius * (self.a.z * theta2.cos() + self.b.z * theta2.sin());

        Bounds::new(z2, z1)
    }

    pub fn collides_with(&self, other: &BoundingShape) -> bool {
        let distance = ((self.center.x - other.center.x).powi(2) +
                            (self.center.y - other.center.y).powi(2) +
                            (self.center.z - other.center.z).powi(2))
            .sqrt();
        distance < (self.max_radius + other.max_radius)
    }
}
