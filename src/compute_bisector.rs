extern crate std;
extern crate rgsl;
extern crate itertools;
use self::itertools::cons_tuples;


pub fn compute_bisector(rv: &[f64], profile: &[f64]) -> Vec<f64> {
    let max_value = profile.iter().cloned().fold(std::f64::NAN, f64::max);

    let (blue_profile, blue_rv): (Vec<f64>, Vec<f64>) = cons_tuples(profile.iter()
        .zip(profile.iter().skip(1))
        .zip(rv.iter()))
        .skip_while(|&(this, _, _)| *this != max_value)
        .take_while(|&(this, next, _)| *this < *next)
        .map(|(this, _, rv)| (this, rv))
        .unzip();

    let (mut red_profile, mut red_rv): (Vec<f64>, Vec<f64>) = cons_tuples(profile.iter().rev()
        .zip(profile.iter().rev().skip(1))
        .zip(rv.iter()))
        .skip_while(|&(this, _, _)| *this != max_value)
        .take_while(|&(this, next, _)| *this < *next)
        .map(|(this, _, rv)| (this, rv))
        .unzip();

    red_profile.reverse();
    red_rv.reverse();

    let mut red_acc = rgsl::InterpAccel::new();
    let red_spline = rgsl::Spline::new(&rgsl::InterpType::cspline(), red_rv.len()).unwrap();
    red_spline.init(&red_profile, &red_rv);

    let mut blue_acc = rgsl::InterpAccel::new();
    let blue_spline = rgsl::Spline::new(&rgsl::InterpType::cspline(), blue_rv.len()).unwrap();
    blue_spline.init(&blue_profile, &blue_rv);

    let bis_size: usize = 1000;
    (0..bis_size)
        .map(|x| x as f64 / bis_size as f64)
        .map(|ccf| (red_spline.eval(ccf, &mut red_acc) + blue_spline.eval(ccf, &mut blue_acc))/2.0)
        .collect::<Vec<f64>>()
}