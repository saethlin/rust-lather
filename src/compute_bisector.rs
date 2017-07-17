extern crate std;
extern crate rgsl;
extern crate itertools;
extern crate gnuplot;
use self::itertools::cons_tuples;
use linspace::linspace;


/// CCFs point down for the purpose fo this function
pub fn compute_bisector(rv: &[f64], profile: &[f64]) -> Vec<f64> {
    let (min_index, _) = profile.iter().enumerate()
        .fold((0, std::f64::NAN),
        |(min_ind, min_val), (current_ind, current_val)| {
        if min_val < *current_val {
            (min_ind, min_val)
        }
        else {
            (current_ind, *current_val)
        }
    });

    let (right_profile, right_rv): (Vec<f64>, Vec<f64>) =
        cons_tuples(profile.iter().zip(profile.iter().skip(1)).zip(rv.iter()))
            .skip(min_index)
            .take_while(|&(this, next, _)| *this <= *next)
            .map(|(this, _, rv)| (this, rv))
            .unzip();

    let (left_profile, left_rv): (Vec<f64>, Vec<f64>) =
        cons_tuples(profile.iter().rev().zip(profile.iter().rev().skip(1)).zip(
            rv.iter().rev(),
        )).skip(profile.len()-min_index)
            .take_while(|&(this, next, _)| *this <= *next)
            .map(|(this, _, rv)| (this, rv))
            .unzip();

    let mut left_acc = rgsl::InterpAccel::new();
    let left_spline = rgsl::Spline::new(&rgsl::InterpType::cspline(), left_rv.len()).unwrap();
    left_spline.init(&left_profile, &left_rv);
    let mut right_acc = rgsl::InterpAccel::new();
    let right_spline = rgsl::Spline::new(&rgsl::InterpType::cspline(), right_rv.len()).unwrap();
    right_spline.init(&right_profile, &right_rv);

    // TODO: this needs an off-by-one to permit interpolation, do I need a shift by one on the other end too?
    let left_max = left_profile[left_profile.len()-2];
    let right_max = right_profile[right_profile.len()-2];
    let bis_start = f64::min(left_profile[1], right_profile[1]);
    linspace(bis_start, f64::min(left_max, right_max), 100)
        .map(|ccf| {
            (left_spline.eval(ccf, &mut left_acc) + right_spline.eval(ccf, &mut right_acc)) / 2.0
        })
        .collect::<Vec<f64>>()
}

#[cfg(test)]
mod tests {
    extern crate itertools;
    use super::*;
    use linspace::linspace;

    fn is_close(actual: f64, expected: f64) -> bool {
        let precision = 5;
        let pow = 10.0.powi(precision+1);
        let delta = (expected - actual).abs();
        let max_delta = 10.0.powi(-precision)/2.0;
        return (delta * pow).round() / pow <= max_delta;
    }

    #[test]
    fn zero_centered_gaussian() {
        let test_len = 101;
        let rv: Vec<f64> = linspace(-1.0, 1.0, test_len).collect();
        let ccf: Vec<f64> = rv.iter().map(|x| -(-x * x).exp()).collect();
        for bis in compute_bisector(&rv, &ccf).iter() {
            /*
            assert!(bis.abs() < 2e-5,
                "zero-centered bisector value is {} but should be no more than {}",
                *bis,
                2e-5
            );
            */
            assert!(is_close(*bis, 0.0), "zero-centered bisector value {} is not close enough to 0", bis);
        }
    }
}
