use rulinalg::matrix::Matrix;
use rulinalg::matrix::BaseMatrix;
use std;
use dim::si::{MeterPerSecond, Unitless, MPS};

pub fn fit_rv(rv: &[MeterPerSecond<f64>], ccf: &[Unitless<f64>]) -> MeterPerSecond<f64> {
    let (min_index, _) = ccf.iter().enumerate().fold(
        (0, Unitless::new(std::f64::INFINITY)),
        |(min_ind, min_val), (current_ind, current_val)| {
            if *current_val < min_val {
                (current_ind, *current_val)
            } else {
                (min_ind, min_val)
            }
        },
    );

    let peak_rv = rv.iter().skip(min_index - 3).take(7).cloned();
    let peak_ccf: Vec<_> = ccf.iter().skip(min_index - 3).take(7).cloned().collect();
    let rv_matrix_values: Vec<_> = std::iter::repeat(1.0 * MPS)
        .take(7)
        .chain(peak_rv.clone())
        .chain(peak_rv.map(|x| x * x))
        .collect();

    let rv_matrix_transpose = Matrix::new(3, 7, rv_matrix_values.clone());
    let rv_matrix = rv_matrix_transpose.transpose();
    let ccf_matrix = Matrix::new(7, 1, peak_ccf);
    let coefficients = (rv_matrix_transpose.clone() * rv_matrix.clone())
        .inverse()
        .unwrap() * rv_matrix_transpose * ccf_matrix;

    -coefficients[[1, 0]] / (2.0 * coefficients[[2, 0]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_close(actual: f64, expected: f64) -> bool {
        let precision = 5;
        let pow = 10.0f64.powi(precision + 1);
        let delta = (expected - actual).abs();
        let max_delta = 10.0f64.powi(-precision) / 2.0;
        return (delta * pow).round() / pow <= max_delta;
    }

    #[test]
    fn simple_gaussian() {
        use linspace::linspace;

        let test_len = 101;
        let rv: Vec<f64> = linspace(-1.0, 1.0, test_len).collect();
        let ccf: Vec<f64> = rv.iter().map(|x| -(-(x - 0.5).powi(2)).exp()).collect();

        let poly_rv = fit_rv(&rv, &ccf);
        assert!(is_close(poly_rv, 0.5));
    }
}
