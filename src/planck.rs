extern crate quadrature;
use self::quadrature::clenshaw_curtis::integrate;

fn planck(wavelength: f64, temperature: f64) -> f64 {
    let exp_m1 = f64::exp_m1;
    let c: f64 = 299792458.0;
    let h = 6.62606896e-34;
    let k_b = 1.380e-23;
    2.0 * h * c.powi(2) * 1.0 /
        (wavelength.powi(5) * (exp_m1((h * c) / (wavelength * k_b * temperature))))
}

pub fn planck_integral(temperature: f64, wave_min: f64, wave_max: f64) -> f64 {
    let func = |wavelength| planck(wavelength, temperature);
    if wave_min < wave_max {
        integrate(func, wave_min, wave_max, 1e-7).integral
    } else {
        integrate(func, wave_max, wave_min, 1e-7).integral
    }
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
    fn test_accuracy() {
        assert!(is_close(
            planck_integral(5778.0, 4000e-10, 7000e-10),
            7359875.725388271,
        ));
    }

    #[test]
    fn test_order_proof() {
        assert_eq!(
            planck_integral(5778.0, 4000e-10, 7000e-10),
            planck_integral(5778.0, 7000e-10, 4000e-10)
        );
    }
}
