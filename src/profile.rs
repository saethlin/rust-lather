extern crate std;
use std::iter;

pub struct Profile {
    pub rv: Vec<f64>,
    pub ccf: Vec<f64>,
    derivative: Vec<f64>,
}


impl Profile {
    pub fn new(rv: Vec<f64>, ccf: Vec<f64>) -> Self {
        let ccf_diff = ccf
            .iter()
            .zip(ccf.iter().skip(1))
            .map(|(a,b) |a-b);
        let rv_diff = rv
            .iter()
            .zip(rv.iter().skip(1))
            .map(|(a,b)| a-b);

        let der = ccf_diff
                .zip(rv_diff)
                .map(|(c, r)| c/r)
                .chain(std::iter::once(0.0))
                .collect::<Vec<f64>>();

        Profile {
            rv: rv.clone(),
            ccf: ccf.clone(),
            derivative: der,
        }
    }

    pub fn len(&self) -> usize {
        self.rv.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn shift(&self, velocity: f64) -> Vec<f64> {
        let stepsize = (self.rv[0] - self.rv[1]).abs();
        let quotient = (velocity / stepsize).round() as isize;
        let remainder = velocity - (quotient as f64) * stepsize;

        if velocity >= 0.0 {
            iter::repeat(self.ccf[0])
                .take(quotient as usize)
                .chain(self.ccf.iter().zip(self.derivative.iter())
                    .take(self.ccf.len() - quotient as usize)
                    .map(|(ccf, der)| ccf - remainder * der)
                ).collect::<Vec<f64>>()
        }
        else {
            self.ccf.iter().zip(self.derivative.iter())
            .skip((-quotient) as usize)
            .map(|(ccf, der)| ccf - remainder * der)
            .chain(iter::repeat(self.ccf[0]).take((-quotient) as usize))  // TODO is this +1 correct?
            .collect::<Vec<f64>>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sun_ccfs::*;

    #[test]
    fn derivative_len_test() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        assert_eq!(test_profile.derivative.len(), ccf_quiet().len(),
        "derivative length is {} but should be {}", test_profile.derivative.len(), ccf_quiet().len());
    }

    #[test]
    fn zero_shift_len_test() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(0.0);
        assert_eq!(shifted.len(), ccf_quiet().len(),
        "zero-shifted length is {} but should be {}", shifted.len(), ccf_quiet().len());
    }

    #[test]
    fn neg_zero_shift_len_test() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(0.0);
        assert_eq!(shifted.len(), ccf_quiet().len(),
        "negative zero-shifted length is {} but should be {}", shifted.len(), ccf_quiet().len());
    }

    #[test]
    fn shift_pos_len_test() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(1.5);
        assert_eq!(shifted.len(), ccf_quiet().len(),
        "postive-shifted length is {} but should be {}", shifted.len(), ccf_quiet().len());
    }

    #[test]
    fn shift_neg_len_test() {
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(-1.5);
        assert_eq!(shifted.len(), ccf_quiet().len(),
        "negative-shifted length is {} but should be {}", shifted.len(), ccf_quiet().len());
    }

    #[test]
    fn zero_shift_pos_test(){
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(0.0);
        for (original, shifted) in ccf_quiet().iter().zip(shifted.iter()) {
            assert_eq!(original, shifted,
            "zero-shifted profile value should is {} but should be {}", original, shifted);
        }
    }

    #[test]
    fn zero_shift_neg_test(){
        let test_profile = Profile::new(rv(), ccf_quiet());
        let shifted = test_profile.shift(-0.0);
        for (original, shifted) in ccf_quiet().iter().zip(shifted.iter()) {
            assert_eq!(original, shifted,
            "negative zero-shifted profile value should is {} but should be {}", original, shifted);
        }
    }
}