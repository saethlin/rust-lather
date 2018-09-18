use itertools::cons_tuples;
use linspace::linspace;
use rgsl;
use std::f64;

pub fn compute_bisector(rv: &[f64], profile: &[f64]) -> Vec<f64> {
    let (min_index, min_value) = profile.iter().enumerate().fold(
        (0, f64::INFINITY),
        |(min_ind, min_val), (current_ind, current_val)| {
            if *current_val < min_val {
                (current_ind, *current_val)
            } else {
                (min_ind, min_val)
            }
        },
    );

    let right_count = profile
        .iter()
        .zip(profile.iter().skip(1))
        .skip(min_index)
        .take_while(|&(this, next)| this <= next)
        .count();
    let right_profile = &profile[min_index..min_index + right_count];
    let right_rv = &rv[min_index..min_index + right_count];

    let mut right_acc = rgsl::InterpAccel::new();
    let right_spline = rgsl::Spline::new(&rgsl::InterpType::cspline(), right_rv.len()).unwrap();
    right_spline.init(right_profile, right_rv);

    // Can't just use slices here because we need to provide the data in revese order for GSL
    let (left_profile, left_rv): (Vec<f64>, Vec<f64>) = cons_tuples(
        profile
            .iter()
            .rev()
            .zip(profile.iter().rev().skip(1))
            .zip(rv.iter().rev()),
    ).skip(profile.len() - min_index - 1) // off-by-one correction to make sure both sides pick up the peak point
    .take_while(|&(this, next, _)| *this <= *next)
    .map(|(this, _, rv)| (this, rv))
    .unzip();

    let mut left_acc = rgsl::InterpAccel::new();
    let left_spline = rgsl::Spline::new(&rgsl::InterpType::cspline(), left_rv.len()).unwrap();
    left_spline.init(&left_profile, &left_rv);

    let left_max = left_profile[left_profile.len() - 1];
    let right_max = right_profile[right_profile.len() - 1];
    let bis_end = f64::min(left_max, right_max);

    let mut output = Vec::with_capacity(1000);
    output.extend(linspace(min_value, bis_end, 1000).map(|ccf| {
        (left_spline.eval(ccf, &mut left_acc) + right_spline.eval(ccf, &mut right_acc)) / 2.0
    }));
    output
}

#[cfg(test)]
mod tests {
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
            assert!(
                is_close(*bis, 0.5),
                "zero-centered bisector value {} is not close enough to 0",
                bis
            );
        }
    }
}
