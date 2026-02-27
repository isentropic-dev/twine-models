//! Isentropic efficiency type for turbomachinery models.

use thiserror::Error;
use uom::si::{f64::Ratio, ratio};

/// Isentropic efficiency for turbomachinery components, constrained to `(0, 1]`.
///
/// For a compressor, represents the ratio of ideal to actual specific work.
/// For a turbine, represents the ratio of actual to ideal specific work.
///
/// Validated at construction — the type carries the guarantee that the
/// efficiency is physically valid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IsentropicEfficiency(Ratio);

/// Error returned when constructing an [`IsentropicEfficiency`] with an out-of-range value.
#[derive(Debug, Error)]
#[error("isentropic efficiency must be in (0, 1], got {value}")]
pub struct InvalidIsentropicEfficiency {
    pub value: f64,
}

impl IsentropicEfficiency {
    /// Constructs an `IsentropicEfficiency` from a dimensionless float.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidIsentropicEfficiency`] if `eta` is not in `(0, 1]`
    /// (including zero, negative values, values greater than 1, and NaN).
    pub fn new(eta: f64) -> Result<Self, InvalidIsentropicEfficiency> {
        if !(eta > 0.0 && eta <= 1.0) {
            return Err(InvalidIsentropicEfficiency { value: eta });
        }
        Ok(Self(Ratio::new::<ratio::ratio>(eta)))
    }

    /// Constructs an `IsentropicEfficiency` from a [`Ratio`].
    ///
    /// # Errors
    ///
    /// Returns [`InvalidIsentropicEfficiency`] if the ratio is not in `(0, 1]`.
    pub fn from_ratio(eta: Ratio) -> Result<Self, InvalidIsentropicEfficiency> {
        let value = eta.get::<ratio::ratio>();
        if !(value > 0.0 && value <= 1.0) {
            return Err(InvalidIsentropicEfficiency { value });
        }
        Ok(Self(eta))
    }

    /// Returns the efficiency as a [`Ratio`].
    #[must_use]
    pub fn ratio(self) -> Ratio {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_range() {
        assert!(IsentropicEfficiency::new(0.01).is_ok());
        assert!(IsentropicEfficiency::new(0.9).is_ok());
        assert!(IsentropicEfficiency::new(1.0).is_ok());
    }

    #[test]
    fn rejects_zero() {
        assert!(IsentropicEfficiency::new(0.0).is_err());
    }

    #[test]
    fn rejects_negative() {
        assert!(IsentropicEfficiency::new(-0.1).is_err());
    }

    #[test]
    fn rejects_above_one() {
        assert!(IsentropicEfficiency::new(1.01).is_err());
    }

    #[test]
    fn rejects_nan() {
        assert!(IsentropicEfficiency::new(f64::NAN).is_err());
    }

    #[test]
    fn from_ratio_roundtrip() {
        let r = Ratio::new::<ratio::ratio>(0.85);
        let eta = IsentropicEfficiency::from_ratio(r).unwrap();
        assert_eq!(eta.ratio(), r);
    }
}
