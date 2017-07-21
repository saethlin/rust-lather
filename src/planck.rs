extern crate rgsl;

fn planck(wavelength: f64, p: &mut f64) -> f64 {
    let exp_m1 = f64::exp_m1;
    let c: f64 = 299792458.0;
    let h = 6.62606896e-34;
    let k_b = 1.380e-23;
    2.0 * h * c.powi(2) * 1.0 / (wavelength.powi(5) * (exp_m1((h * c) / (wavelength * k_b * (*p)))))
}

pub fn planck_integral(temperature: f64, wave_min: f64, wave_max: f64) -> f64 {
    let mut t = temperature;
    let mut result: f64 = 0.0;
    let mut error: f64 = 0.0;
    let mut w = rgsl::IntegrationWorkspace::new(1000).unwrap();
    w.qags(
        planck,
        &mut t,
        wave_min,
        wave_max,
        0f64,
        1e-7f64,
        1000,
        &mut result,
        &mut error,
    );

    result
}
