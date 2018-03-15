use num_complex::Complex;
use rustfft;

#[allow(dead_code)]
pub fn set_resolution(rv: &[f64], ccf: &[f64]) -> Vec<f64> {
    let sqrt = f64::sqrt;
    let ln = f64::ln;
    let exp = f64::exp;

    let c = 299792458.0;
    let resolution = 1e5;
    let profile_fwhm = c / resolution;
    let profile_sigma = profile_fwhm / (2.0 * sqrt(2.0 * ln(2.0)));
    let kernel: Vec<Complex<f64>> = rv.iter()
        .map(|r| exp(-r.powi(2) / (2.0 * profile_sigma.powi(2))))
        .map(|n| Complex::new(n, 0.0))
        .collect();

    // FFT convolution
    let fft = rustfft::FFTplanner::new(false).plan_fft(rv.len());
    let inverse_fft = rustfft::FFTplanner::new(true).plan_fft(rv.len());

    // FFT of the kernel
    let mut fft_kernel = vec![Complex::new(0.0, 0.0); rv.len()];
    fft.process(&mut kernel.clone(), fft_kernel.as_mut_slice());

    // FFT of the CCF
    let mut fft_ccf = vec![Complex::new(0.0, 0.0); rv.len()];
    let mut ccf_complex: Vec<_> = ccf.iter().map(|c| Complex::new(*c, 0.0)).collect();
    fft.process(&mut ccf_complex, fft_ccf.as_mut_slice());

    // Multiply both frequencies and compute inverse transform
    let mut convolved_fft: Vec<Complex<f64>> = fft_kernel
        .iter()
        .rev()
        .zip(fft_ccf.iter())
        .map(|(k, c)| c * k)
        .collect();

    let mut output = vec![Complex::new(0.0, 0.0); rv.len()];
    inverse_fft.process(&mut convolved_fft, output.as_mut_slice());

    output.iter().map(|n| n.re).collect::<Vec<f64>>()
}

/*
c = 299792458. # speed of light in m/s
HARPS_resolution = inst_reso
HARPS_inst_profile_FWHM  = c/HARPS_resolution/1000.
HARPS_inst_profile_sigma = HARPS_inst_profile_FWHM/(2*sqrt(2*log(2)))
Gaussian_low_reso = exp(-ccf.rv**2/(2*(HARPS_inst_profile_sigma)**2))

CCFstar_quiet_tmp = signal.convolve(-CCFstar_quiet+1,Gaussian_low_reso,'same')
CCFstar_quiet = 1-CCFstar_quiet_tmp*(1-min(CCFstar_quiet))/max(CCFstar_quiet_tmp)
*/
