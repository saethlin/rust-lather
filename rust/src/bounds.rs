/// Boundaries or the extent of some quantity
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub lower: f64,
    pub upper: f64,
}

impl Bounds {
    /// Constructs a Bounds from any two values, ensuring that the lesser
    /// is assigned to lower, and the greater to upper
    pub fn new(val1: f64, val2: f64) -> Self {
        if val1 <= val2 {
            Bounds {
                lower: val1,
                upper: val2,
            }
        } else {
            Bounds {
                lower: val2,
                upper: val1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_order_test() {
        let bounds = Bounds::new(0.0, 1.0);
        assert_eq!(
            bounds.lower, 0.0,
            "lower value is {} but should be {}",
            bounds.lower, 0.0
        );
        assert_eq!(
            bounds.upper, 1.0,
            "lower value is {} but should be {}",
            bounds.upper, 1.0
        );
    }

    #[test]
    fn out_of_order_test() {
        let bounds = Bounds::new(1.0, 0.0);
        assert_eq!(
            bounds.lower, 0.0,
            "lower value is {} but should be {}",
            bounds.lower, 0.0
        );
        assert_eq!(
            bounds.upper, 1.0,
            "lower value is {} but should be {}",
            bounds.upper, 1.0
        );
    }
}
