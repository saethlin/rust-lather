extern crate std;

#[inline]
pub fn linspace(start: f64, stop: f64, len: usize) -> Linspace {
    Linspace {
        start: start,
        len: len,
        step: (stop - start) / (len as f64),
        index: 0,
    }
}

pub struct Linspace {
    start: f64,
    len: usize,
    step: f64,
    index: usize,
}

impl Iterator for Linspace {
    type Item = f64;
    #[inline]
    fn next(&mut self) -> Option<f64> {
        if self.index >= self.len {
            None
        } else {
            let output = Some(self.start + self.step * (self.index as f64));
            self.index += 1;
            output
        }
    }
}
