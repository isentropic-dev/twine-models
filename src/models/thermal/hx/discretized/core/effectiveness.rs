use std::{error::Error, fmt};

/// Heat exchanger effectiveness (dimensionless, 0 to 1).
///
/// The ratio of actual heat transfer to the maximum possible
/// heat transfer given the operating conditions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effectiveness(f64);

/// Error returned when an effectiveness value is out of range.
#[derive(Debug, Clone, Copy)]
pub struct EffectivenessError {
    value: f64,
}

impl fmt::Display for EffectivenessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "effectiveness must be in [0, 1], got {}", self.value)
    }
}

impl Error for EffectivenessError {}

impl Effectiveness {
    /// Creates an effectiveness value.
    ///
    /// # Errors
    ///
    /// Returns [`EffectivenessError`] if `value` is not in `[0.0, 1.0]`.
    pub fn new(value: f64) -> Result<Self, EffectivenessError> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(EffectivenessError { value })
        }
    }

    /// Returns the effectiveness as a dimensionless scalar.
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    #[test]
    fn zero_and_one_are_valid() {
        assert_relative_eq!(Effectiveness::new(0.0).unwrap().get(), 0.0);
        assert_relative_eq!(Effectiveness::new(1.0).unwrap().get(), 1.0);
    }

    #[test]
    fn midpoint_is_valid() {
        assert_relative_eq!(Effectiveness::new(0.5).unwrap().get(), 0.5);
    }

    #[test]
    fn negative_is_rejected() {
        assert!(Effectiveness::new(-0.1).is_err());
    }

    #[test]
    fn above_one_is_rejected() {
        assert!(Effectiveness::new(1.1).is_err());
    }
}
