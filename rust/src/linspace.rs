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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.len - self.index))
    }
}

impl ExactSizeIterator for Linspace {
    fn len(&self) -> usize {
        self.len
    }
}

/// Creates an iterator of `len` floats between `start`
/// and `stop`, inclusive.
pub fn linspace(start: f64, stop: f64, len: usize) -> Linspace {
    Linspace {
        start: start,
        len: len,
        step: (stop - start) / (len as f64 - 1.0),
        index: 0,
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
        if self.index > self.len {
            None
        } else {
            let output = Some(self.start + self.step * (self.index as f64));
            self.index += 1;
            output
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.len - self.index))
    }
}

impl ExactSizeIterator for Floatrange {
    fn len(&self) -> usize {
        self.len
    }
}

/// Creates an iterator of floats that begin at `start` and
/// increase by `step` until they would exceed `stop`.
pub fn floatrange(start: f64, stop: f64, step: f64) -> Floatrange {
    let len = ((stop - start) / step) as usize;
    Floatrange {
        index: 0,
        len: len,
        start: start,
        step: step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linspace_len() {
        assert_eq!(linspace(0.0, 10.0, 10).count(), 10);
    }

    #[test]
    fn floatrange_len() {
        assert_eq!(floatrange(0.0, 10.0, 1.0).count(), 11);
    }

    #[test]
    fn linspace_empty() {
        assert!(linspace(0.0, 10.0, 0).next().is_none());
        assert_eq!(linspace(0.0, 0.0, 100).count(), 100);
    }

    #[test]
    fn floatrange_empty() {
        assert_eq!(floatrange(0.0, 10.0, 11.0).count(), 1);
        assert_eq!(floatrange(0.0, 0.0, 10.0).count(), 1);
    }

    #[test]
    fn linspace_bounds() {
        assert_eq!(linspace(0.0, 10.0, 10).next().unwrap(), 0.0);
        assert_eq!(linspace(0.0, 10.0, 10).last().unwrap(), 10.0);
    }

    #[test]
    fn floatrange_bounds() {
        assert_eq!(floatrange(0.0, 10.0, 1.0).next().unwrap(), 0.0);
        assert_eq!(floatrange(0.0, 10.0, 1.0).last().unwrap(), 10.0);
    }

    #[test]
    fn basic_floatrange() {
        let mut frange = floatrange(0.0, 2.0, 1.0);
        assert_eq!(frange.next().unwrap(), 0.0);
        assert_eq!(frange.next().unwrap(), 1.0);
        assert_eq!(frange.next().unwrap(), 2.0);
        assert!(frange.next().is_none());
    }

    #[test]
    fn basic_linspace() {
        let mut lspace = linspace(0.0, 2.0, 3);
        assert_eq!(lspace.next().unwrap(), 0.0);
        assert_eq!(lspace.next().unwrap(), 1.0);
        assert_eq!(lspace.next().unwrap(), 2.0);
        assert!(lspace.next().is_none());
    }
}
