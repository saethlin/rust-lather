#[cfg(feature = "simd")]
use faster::*;

/// A cross-correlation profile, which can be shifted by fast linear interpolation.
#[derive(Debug)]
pub struct Profile {
    pub rv: Vec<f64>,
    pub ccf: Vec<f64>,
    derivative: Vec<f64>,
    stepsize: f64,
}

impl Profile {
    /// Creates a profile with the provided radial velocity and
    /// cross-correlation function, and computes the derivative to enable fast
    /// linear interpolation to simulate viewing the profile at different
    /// relative velocities.
    #[cfg(feature = "simd")]
    pub fn new(rv: Vec<f64>, ccf: Vec<f64>) -> Self {
        let mut derivative = vec![0.0; ccf.len()];
        (
            ccf[..ccf.len() - 1].simd_iter(f64s(0.0)),
            ccf[1..].simd_iter(f64s(0.0)),
            rv[..rv.len() - 1].simd_iter(f64s(0.0)),
            rv[1..].simd_iter(f64s(0.0)),
        )
            .zip()
            .simd_map(|(cl, cr, rvl, rvr)| (cl - cr) / (rvl - rvr))
            .scalar_fill(&mut derivative);

        let stepsize = (rv[0] - rv[1]).abs();

        Profile {
            rv,
            ccf,
            derivative,
            stepsize,
        }
    }

    #[cfg(not(feature = "simd"))]
    pub fn new(rv: Vec<f64>, ccf: Vec<f64>) -> Self {
        use std::iter;

        let derivative = {
            let ccf_diff = ccf.windows(2).map(|s| s[0] - s[1]);
            let rv_diff = rv.windows(2).map(|s| s[0] - s[1]);

            ccf_diff
                .zip(rv_diff)
                .map(|(c, r)| c / r)
                .chain(iter::once(0.0))
                .collect()
        };

        let stepsize = (rv[0] - rv[1]).abs();

        Profile {
            rv,
            ccf,
            derivative,
            stepsize,
        }
    }

    /// Returns the number of elements in the profile.
    pub fn len(&self) -> usize {
        self.rv.len()
    }

    /// Uses the pre-computed derivative to compute a shifted version of
    /// this profile's cross-correlation function by linear interpolation.
    /// The units of velocity must match those of the radial velocity used
    /// to construct this profile.
    #[cfg(feature = "simd")]
    pub fn shift_into(&self, velocity: f64, output: &mut [f64]) {
        let quotient = (velocity / self.stepsize).round() as isize;
        let remainder = velocity - (quotient as f64) * self.stepsize;

        if velocity >= 0.0 {
            let quotient = quotient as usize;
            for i in 0..quotient {
                output[i] = 0.0;
            }

            (
                self.ccf[..output.len() - quotient].simd_iter(f64s(0.0)),
                self.derivative[..output.len() - quotient].simd_iter(f64s(0.0)),
            )
                .zip()
                .simd_map(|(ccf, der)| ccf - f64s(remainder) * der)
                .scalar_fill(&mut output[quotient..]);
        } else {
            let quot = -quotient as usize;
            let len = output.len();
            (
                self.ccf[quot..].simd_iter(f64s(0.0)),
                self.derivative[quot..].simd_iter(f64s(0.0)),
            )
                .zip()
                .simd_map(|(ccf, der)| ccf - f64s(remainder) * der)
                .scalar_fill(&mut output[..len - quot]);

            for i in (output.len() - quot)..output.len() {
                output[i] = 0.0;
            }
        }
    }

    #[cfg(not(feature = "simd"))]
    pub fn shift_into(&self, velocity: f64, output: &mut [f64]) {
        use std::iter;
        let quotient = (velocity / self.stepsize).round() as isize;
        let remainder = velocity - (quotient as f64) * self.stepsize;

        if velocity >= 0.0 {
            iter::repeat(self.ccf[0])
                .take(quotient as usize)
                .chain(
                    self.ccf
                        .iter()
                        .zip(self.derivative.iter())
                        .take(self.ccf.len() - quotient as usize)
                        .map(|(ccf, der)| ccf - remainder * der),
                ).zip(output.iter_mut())
                .for_each(|(shifted, output)| *output = shifted)
        } else {
            self.ccf
                .iter()
                .zip(self.derivative.iter())
                .skip((-quotient) as usize)
                .map(|(ccf, der)| ccf - remainder * der)
                .chain(iter::repeat(self.ccf[0]).take((-quotient) as usize))
                .zip(output.iter_mut())
                .for_each(|(shifted, output)| *output = shifted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solar_ccfs::*;

    #[test]
    fn derivative_len() {
        let test_profile = Profile::new(RV.to_vec(), CCF_QUIET.to_vec());
        assert_eq!(
            test_profile.derivative.len(),
            CCF_QUIET.len(),
            "derivative length is {} but should be {}",
            test_profile.derivative.len(),
            CCF_QUIET.len()
        );
    }

    #[test]
    fn pos_zero_shift() {
        let test_profile = Profile::new(RV.to_vec(), CCF_QUIET.to_vec());
        let mut shifted = vec![0.0; test_profile.len()];
        test_profile.shift_into(0.0, &mut shifted);
        for (original, shifted) in CCF_QUIET.iter().zip(shifted.iter()) {
            assert_eq!(
                original, shifted,
                "zero-shifted profile value is {} but should be {}",
                original, shifted
            );
        }
    }

    #[test]
    fn neg_zero_shift() {
        let test_profile = Profile::new(RV.to_vec(), CCF_QUIET.to_vec());
        let mut shifted = vec![0.0; test_profile.len()];
        test_profile.shift_into(-0.0, &mut shifted);
        for (original, shifted) in CCF_QUIET.iter().zip(shifted.iter()) {
            assert_eq!(
                original, shifted,
                "negative zero-shifted profile value is {} but should be {}",
                original, shifted
            );
        }
    }
}
