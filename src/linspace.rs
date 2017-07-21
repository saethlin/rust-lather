extern crate std;

pub fn linspace(start: f64, stop: f64, len: usize) -> Linspace {
    Linspace {
        start: start,
        len: len,
        step: (stop - start) / (len as f64),
        index: 0,
    }
}

pub struct Linspace {
    index: usize,
    len: usize,
    start: f64,
    step: f64,
}

impl Iterator for Linspace {
    type Item = f64;
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

pub fn floatrange(start: f64, stop: f64, step: f64) -> Floatrange {
    Floatrange {
        index: 0,
        len: ((stop - start) / step) as usize,
        start: start,
        step: step,
    }
}

pub struct Floatrange {
    index: usize,
    len: usize,
    start: f64,
    step: f64,
}

impl Iterator for Floatrange {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        if self.index == self.len {
            None
        } else {
            let output = Some(self.start + self.step * (self.index as f64));
            self.index += 1;
            output
        }
    }
}
