#[derive(PartialEq, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[allow(dead_code)]
impl Point {
    pub fn rotated_x(&self, angle: f64) -> Point {
        Point {
            x: self.x,
            y: self.y * angle.sin() - self.z * angle.sin(),
            z: self.y * angle.sin() + self.z * angle.cos(),
        }
    }

    pub fn rotated_y(&self, angle: f64) -> Point {
        Point {
            x: self.z * angle.sin() + self.x * angle.cos(),
            y: self.y,
            z: self.z * angle.cos() - self.x * angle.sin(),
        }
    }

    pub fn rotated_z(&self, angle: f64) -> Point {
        Point {
            x: self.x * angle.cos() - self.y * angle.sin(),
            y: self.x * angle.sin() + self.y * angle.cos(),
            z: self.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts;

    fn is_close(actual: f64, expected: f64) -> bool {
        let precision = 15;
        let pow = 10.0f64.powi(precision + 1);
        let delta = (expected - actual).abs();
        let max_delta = 10.0f64.powi(-precision) / 2.0;
        return (delta * pow).round() / pow <= max_delta;
    }

    #[test]
    fn x_rotation() {
        let point = Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
        .rotated_x(consts::FRAC_PI_4);
        assert_eq!(point.x, 1.0);
        assert!(is_close(point.y, 0.0));
        assert!(is_close(point.z, consts::SQRT_2));
    }

    #[test]
    fn y_rotation() {
        let point = Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
        .rotated_y(consts::FRAC_PI_4);
        assert!(is_close(point.x, consts::SQRT_2));
        assert_eq!(point.y, 1.0);
        assert!(is_close(point.z, 0.0));
    }

    #[test]
    fn z_rotation() {
        let point = Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
        .rotated_z(consts::FRAC_PI_4);
        assert!(is_close(point.x, 0.0));
        assert!(is_close(point.y, consts::SQRT_2));
        assert_eq!(point.z, 1.0);
    }
}
