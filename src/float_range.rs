pub struct FloatRange(pub f64, pub f64, pub f64);

impl Iterator for FloatRange {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<f64> {
        if self.0 < self.1 {
            let v = self.0;
            self.0 = v + self.2;
            Some(v)
        } else {
            None
        }
    }
}
