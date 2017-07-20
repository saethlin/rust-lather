extern crate std;
use std::iter;

/// A cross-correlation profile, which stores its derivative inline to enable
/// fast linear interpolation
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
    /// relative velocities

    pub fn new(rv: Vec<f64>, ccf: Vec<f64>) -> Self {
        let ccf_diff = ccf.iter().zip(ccf.iter().skip(1)).map(|(a, b)| a - b);
        let rv_diff = rv.iter().zip(rv.iter().skip(1)).map(|(a, b)| a - b);

        let der = ccf_diff
            .zip(rv_diff)
            .map(|(c, r)| c / r)
            .chain(std::iter::once(0.0))
            .collect::<Vec<f64>>();

        Profile {
            rv: rv.clone(),
            ccf: ccf.clone(),
            derivative: der,
            stepsize: (rv[0] - rv[1]).abs(),
        }
    }

    pub fn len(&self) -> usize {
        self.rv.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn shift(&self, velocity: f64) -> Vec<f64> {
        /// Uses the pre-computed derivative to compute a shifted version of
        /// this profile's cross-correlation function by linear interpolation
        ///
        /// The units of velocity must match those of the radial velocity used
        /// to construct this profile
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
                )
                .collect::<Vec<f64>>()
        } else {
            self.ccf.iter().zip(self.derivative.iter())
            .skip((-quotient) as usize)
            .map(|(ccf, der)| ccf - remainder * der)
            .chain(iter::repeat(self.ccf[0]).take((-quotient) as usize))
            .collect::<Vec<f64>>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sun_ccfs::*;

    #[test]
    fn derivative_len() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        assert_eq!(
            test_profile.derivative.len(),
            ccf_quiet().len(),
            "derivative length is {} but should be {}",
            test_profile.derivative.len(),
            ccf_quiet().len()
        );
    }

    #[test]
    fn pos_zero_shift_len() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(0.0);
        assert_eq!(
            shifted.len(),
            ccf_quiet().len(),
            "zero-shifted length is {} but should be {}",
            shifted.len(),
            ccf_quiet().len()
        );
    }

    #[test]
    fn neg_zero_shift_len() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(0.0);
        assert_eq!(
            shifted.len(),
            ccf_quiet().len(),
            "negative zero-shifted length is {} but should be {}",
            shifted.len(),
            ccf_quiet().len()
        );
    }

    #[test]
    fn pos_shift_len() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(1.5);
        assert_eq!(
            shifted.len(),
            ccf_quiet().len(),
            "postive-shifted length is {} but should be {}",
            shifted.len(),
            ccf_quiet().len()
        );
    }

    #[test]
    fn neg_shift_len() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(-1.5);
        assert_eq!(
            shifted.len(),
            ccf_quiet().len(),
            "negative-shifted length is {} but should be {}",
            shifted.len(),
            ccf_quiet().len()
        );
    }

    #[test]
    fn pos_zero_shift() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(0.0);
        for (original, shifted) in ccf_quiet().iter().zip(shifted.iter()) {
            assert_eq!(
                original,
                shifted,
                "zero-shifted profile value is {} but should be {}",
                original,
                shifted
            );
        }
    }

    #[test]
    fn neg_zero_shift() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(-0.0);
        for (original, shifted) in ccf_quiet().iter().zip(shifted.iter()) {
            assert_eq!(
                original,
                shifted,
                "negative zero-shifted profile value is {} but should be {}",
                original,
                shifted
            );
        }
    }
}
