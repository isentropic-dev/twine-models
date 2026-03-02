use uom::{ConstZero, si::f64::Power};

use super::StratifiedTankError;

/// Auxiliary heat flow for a tank heat source or sink.
///
/// Distinguishes heating from cooling by name rather than sign, avoiding
/// the ambiguity of a raw signed power value. Both `Heating` and `Cooling`
/// store a strictly positive magnitude; direction is encoded by the variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuxHeatFlow {
    /// Heat added to the fluid (e.g., an electric heating element).
    ///
    /// The stored power is the rate at which energy enters the fluid.
    /// Guaranteed strictly positive.
    Heating(Power),

    /// Heat removed from the fluid (e.g., a cooling coil).
    ///
    /// The stored power is the rate at which energy leaves the fluid.
    /// Guaranteed strictly positive.
    Cooling(Power),

    /// No auxiliary heat flow.
    None,
}

impl AuxHeatFlow {
    /// Creates a `Heating` variant.
    ///
    /// # Errors
    ///
    /// Returns [`StratifiedTankError::NonPositiveAuxPower`] if `power` is not
    /// strictly positive.
    pub fn heating(power: Power) -> Result<Self, StratifiedTankError> {
        if power <= Power::ZERO {
            return Err(StratifiedTankError::NonPositiveAuxPower(power));
        }
        Ok(Self::Heating(power))
    }

    /// Creates a `Cooling` variant.
    ///
    /// # Errors
    ///
    /// Returns [`StratifiedTankError::NonPositiveAuxPower`] if `power` is not
    /// strictly positive.
    pub fn cooling(power: Power) -> Result<Self, StratifiedTankError> {
        if power <= Power::ZERO {
            return Err(StratifiedTankError::NonPositiveAuxPower(power));
        }
        Ok(Self::Cooling(power))
    }

    /// Returns the signed power: positive for heating, negative for cooling, zero for none.
    pub(super) fn signed(self) -> Power {
        match self {
            Self::Heating(p) => p,
            Self::Cooling(p) => -p,
            Self::None => Power::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::power::watt;

    fn w(v: f64) -> Power {
        Power::new::<watt>(v)
    }

    #[test]
    fn heating_positive_ok() {
        assert!(AuxHeatFlow::heating(w(1.0)).is_ok());
    }

    #[test]
    fn cooling_positive_ok() {
        assert!(AuxHeatFlow::cooling(w(1.0)).is_ok());
    }

    #[test]
    fn heating_zero_errors() {
        assert!(matches!(
            AuxHeatFlow::heating(Power::ZERO),
            Err(StratifiedTankError::NonPositiveAuxPower(_))
        ));
    }

    #[test]
    fn heating_negative_errors() {
        assert!(matches!(
            AuxHeatFlow::heating(w(-1.0)),
            Err(StratifiedTankError::NonPositiveAuxPower(_))
        ));
    }

    #[test]
    fn cooling_zero_errors() {
        assert!(matches!(
            AuxHeatFlow::cooling(Power::ZERO),
            Err(StratifiedTankError::NonPositiveAuxPower(_))
        ));
    }

    #[test]
    fn signed_heating_is_positive() {
        let q = AuxHeatFlow::heating(w(5.0)).unwrap();
        assert!(q.signed() > Power::ZERO);
    }

    #[test]
    fn signed_cooling_is_negative() {
        let q = AuxHeatFlow::cooling(w(5.0)).unwrap();
        assert!(q.signed() < Power::ZERO);
    }

    #[test]
    fn signed_none_is_zero() {
        assert_eq!(AuxHeatFlow::None.signed(), Power::ZERO);
    }
}
