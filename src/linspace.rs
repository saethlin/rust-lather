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
    start: f64,
    len: usize,
    step: f64,
    index: usize,
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
        current: start,
        stop: stop,
        step: step,
    }
}

pub struct Floatrange {
    current: f64,
    stop: f64,
    step: f64,
}

impl Iterator for Floatrange {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        // TODO: There must be a neater way to implement this
        if (self.stop == self.current) || ((self.stop - self.current).is_sign_positive() != self.step.is_sign_positive()) {
            None
        } else {
            let output = Some(self.current);
            self.current += self.step;
            output
        }
    }
}
