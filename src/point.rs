extern crate std;

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

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
