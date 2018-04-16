use std::f64::consts;

use bounds::Bounds;
use point::Point;
use spot::Spot;

pub struct BoundingShape {
    center: Point,
    circle_center: Point,
    a: Point,
    b: Point,
    radius: f64,
    circle_radius: f64,
    max_radius: f64,
    visible: bool,
    //on_edge: bool,
    grid_interval: f64,
}

impl BoundingShape {
    pub fn new(spot: &Spot, time: f64) -> Self {
        let max_radius = spot.radius;
        let radius = if spot.mortal {
            let lifetime = spot.time_disappear - spot.time_appear;
            let growth_time = 0.1 * lifetime;
            if (time - spot.time_appear).abs() < growth_time {
                spot.radius * (time - spot.time_appear).abs() / growth_time
            } else if (time - spot.time_disappear).abs() < growth_time {
                spot.radius * (time - spot.time_disappear).abs() / growth_time
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
        let circle_radius = (radius.powi(2) - (1.0 - depth).powi(2)).sqrt();
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

        let x1 = circle_center.x
            + circle_radius * ((theta_x_max).cos() * a.x + (theta_x_max).sin() * b.x);
        let x2 = circle_center.x
            + circle_radius * ((theta_x_min).cos() * a.x + (theta_x_min).sin() * b.x);

        let grid_interval = 2.0 / spot.star.grid_size as f64;
        let visible = x1 > 0.0 || x2 > 0.0;

        BoundingShape {
            center: center,
            circle_center: circle_center,
            a: a,
            b: b,
            radius: radius,
            circle_radius: circle_radius,
            max_radius: max_radius,
            visible: visible,
            //on_edge: x1 < grid_interval || x2 < grid_interval,
            grid_interval: grid_interval,
        }
    }

    pub fn y_bounds(&self) -> Option<Bounds> {
        if !self.visible {
            return None;
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

        let y_max = self.circle_center.y
            + self.circle_radius * (theta_y_max.cos() * self.a.y + theta_y_max.sin() * self.b.y);
        let y_min = self.circle_center.y
            + self.circle_radius * (theta_y_min.cos() * self.a.y + theta_y_min.sin() * self.b.y);

        let x_max = self.circle_center.x
            + self.circle_radius * (theta_y_max.cos() * self.a.y + theta_y_max.sin() * self.b.y);
        let x_min = self.circle_center.x
            + self.circle_radius * (theta_y_min.cos() * self.a.y + theta_y_min.sin() * self.b.y);

        if x_min < 0.0 && x_max < 0.0 {
            None
        } else {
            Some(Bounds::new(y_min, y_max))
        }
    }

    pub fn z_bounds(&self, y: f64, guess: &mut Option<Bounds>) -> Option<Bounds> {
        if y.abs() >= 1.0 {
            return None;
        }

        let new_bounds = match guess {
            &mut Some(ref the_guess) => self.z_bounds_ansatz(y, &the_guess),
            &mut None => self.z_bounds_brute(y),
        };

        *guess = new_bounds.clone();
        new_bounds
    }

    fn z_bounds_ansatz(&self, y: f64, guess: &Bounds) -> Option<Bounds> {
        use linspace::floatrange;
        let z_max = if !self.on_spot(y, guess.upper) {
            // Guess must be above the edge, walk down until on it
            floatrange(
                guess.upper,
                self.center.z - self.radius,
                -self.grid_interval / 1.0,
            ).find(|z| self.on_spot(y, *z))
        } else {
            // Guess must be below the edge, walk up until not on it
            floatrange(
                guess.upper,
                self.center.z + self.radius,
                self.grid_interval / 1.0,
            ).find(|z| !self.on_spot(y, *z))
        };

        let z_min = if !self.on_spot(y, guess.lower) {
            // Guess must be below the edge, walk up until on it
            floatrange(
                guess.lower,
                self.center.z + self.radius,
                self.grid_interval / 1.0,
            ).find(|z| self.on_spot(y, *z))
        } else {
            // Guess must be above the edge, walk down until not on it
            floatrange(
                guess.lower,
                self.center.z - self.radius,
                -self.grid_interval / 1.0,
            ).find(|z| !self.on_spot(y, *z))
        };

        // NOTE: This function used to have a special case for if the edges of a spot
        // passed to the far hemisphere. Those checks now appear to be unnecessary.
        if let (Some(z_min), Some(z_max)) = (z_min, z_max) {
            Some(Bounds::new(z_min, z_max))
        } else {
            None
        }
    }

    fn z_bounds_brute(&self, y: f64) -> Option<Bounds> {
        use linspace::floatrange;

        let z_max = floatrange(
            self.center.z + self.radius,
            self.center.z - self.radius,
            -self.grid_interval / 1.0,
        ).find(|z| self.on_spot(y, *z));

        let z_min = floatrange(
            self.center.z - self.radius,
            self.center.z + self.radius,
            self.grid_interval / 1.0,
        ).find(|z| self.on_spot(y, *z));

        if z_max.is_none() || z_min.is_none() {
            None
        } else {
            Some(Bounds::new(z_min.unwrap(), z_max.unwrap()))
        }
    }

    pub fn on_spot(&self, y: f64, z: f64) -> bool {
        if !on_star(y, z) {
            return false;
        }
        let x = (1.0 - (y * y + z * z)).sqrt();
        let distance_squared = (y - self.center.y) * (y - self.center.y)
            + (z - self.center.z) * (z - self.center.z)
            + (x - self.center.x) * (x - self.center.x);

        distance_squared <= (self.radius * self.radius)
    }

    pub fn collides_with(&self, other: &BoundingShape) -> bool {
        let distance = ((self.center.x - other.center.x).powi(2)
            + (self.center.y - other.center.y).powi(2)
            + (self.center.z - other.center.z).powi(2))
            .sqrt();
        distance < (self.max_radius + other.max_radius)
    }
}

fn on_star(y: f64, z: f64) -> bool {
    (y * y + z * z) <= 1.0
}
