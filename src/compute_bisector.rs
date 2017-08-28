use linspace::linspace;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

// May be able to make this not own its data in the future
struct Interpolator {
    data: Vec<Point>,
}

impl Interpolator {
    pub fn from(data: Vec<Point>) -> Self {
        Interpolator {
            data: data,
        }
    }

    // Optimization around nearest-neighbor check before attempting binary search
    pub fn eval(&self, y: f64) -> f64 {
        // Check that the requested value is in valid range
        // Could also return an Err state here, but do we want to gum up the API that much?
        // Is this error really recoverable?
        // We also know that y is increasing, but that should probably be checked for
        assert!(y >= self.data[0].y && y <= self.data[self.data.len() - 1].y);

        let location = self.data.binary_search_by(|probe| if probe.y == y {
            Ordering::Equal
        } else if probe.y < y {
            Ordering::Less
        } else {
            Ordering::Greater
        });

        let second_index = match location {
            Ok(t) => return self.data[t].x,
            Err(t) => t,
        };
        let first_index = second_index - 1;

        let first = &self.data[first_index];
        let second = &self.data[second_index];

        let slope = (first.x - second.x) / (first.y - second.y);
        let intercept = first.x - slope * first.y;

        slope * y + intercept
    }
}

pub fn compute_bisector(rv: &[f64], profile: &[f64]) -> Vec<Point> {

    let mut data: Vec<Point> = rv.iter()
        .zip(profile.iter())
        .map(|(x, y)| Point { x: *x, y: *y })
        .collect();

    data.sort_by(|a, b| if a.x < b.x {
        Ordering::Less
    } else {
        Ordering::Greater
    });

    let (min_index, min_value) = data.iter()
        .enumerate()
        .min_by(|a, b| if a.1.y < b.1.y {
            Ordering::Less
        } else {
            Ordering::Greater
        })
        .unwrap();

    // TODO probably don't collect these?
    let right_profile: Vec<Point> = data.iter()
        .zip(data.iter().skip(1))
        .skip(min_index)
        .take_while(|&(this, next)| this.y <= next.y)
        .map(|(this, _)| this.clone())
        .collect();

    let left_profile: Vec<Point> = data.iter()
        .rev()
        .zip(data.iter().rev().skip(1))
        .skip(profile.len() - min_index - 1)
        .take_while(|&(this, next)| this.y <= next.y)
        .map(|(this, _)| this.clone())
        .collect();

    let bis_max = f64::min(
        right_profile.iter().last().unwrap().y,
        left_profile.iter().last().unwrap().y,
    );

    let right_interp = Interpolator::from(right_profile);
    let left_interp = Interpolator::from(left_profile);

    linspace(min_value.y, bis_max, 1000)
        .map(|y| {
            Point {
                x: (left_interp.eval(y) + right_interp.eval(y)) / 2.0,
                y: y,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    extern crate itertools;
    use super::*;
    use linspace::linspace;

    fn is_close(actual: f64, expected: f64) -> bool {
        let precision = 4;
        let pow = 10.0f64.powi(precision + 1);
        let delta = (expected - actual).abs();
        let max_delta = 10.0f64.powi(-precision) / 2.0;
        return (delta * pow).round() / pow <= max_delta;
    }

    #[test]
    fn gaussian() {
        let test_len = 101;
        let rv: Vec<f64> = linspace(-1.0, 1.0, test_len).collect();
        let ccf: Vec<f64> = rv.iter().map(|x| -(-(x - 0.5).powi(2)).exp()).collect();
        for bis in compute_bisector(&rv, &ccf).iter().skip(rv.len() / 10) {
            println!("{:#?}", bis);
            assert!(
                is_close(bis.x, 0.5),
                "zero-centered bisector value {} is not close enough to 0",
                bis.x
            );
        }
    }
}
