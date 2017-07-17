extern crate std;

pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn rotate_x(&mut self, angle: f64) {
        self.y = self.y * angle.sin() - self.z * angle.sin();
        self.z = self.y * angle.sin() + self.z * angle.cos();
    }

    pub fn rotate_y(&mut self, angle: f64) {
        self.z = self.z * angle.cos() - self.x * angle.sin();
        self.x = self.z * angle.sin() + self.x * angle.cos();
    }

    pub fn rotate_z(&mut self, angle: f64) {
        self.x = self.x * angle.cos() - self.y * angle.sin();
        self.y = self.x * angle.sin() + self.y * angle.cos();
    }
}
