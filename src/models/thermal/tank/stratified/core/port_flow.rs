use uom::si::f64::{ThermodynamicTemperature, VolumeRate};

use super::StratifiedTankError;

/// Inlet conditions for a port pair.
///
/// A port pair models one real-world hydraulic circuit connection.  Fluid
/// enters the tank at [`inlet_temperature`](PortFlow::inlet_temperature) at
/// the port's inlet location and leaves at the node temperature at the
/// outlet location, at the same volumetric [`rate`](PortFlow::rate).
///
/// # Validation
///
/// Use [`PortFlow::new`] to construct — it validates that the flow rate is
/// non-negative and finite, and that the inlet temperature is finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortFlow {
    rate: VolumeRate,
    /// Inlet fluid temperature.
    ///
    /// The outlet temperature is determined by the node(s) at the outlet location.
    pub inlet_temperature: ThermodynamicTemperature,
}

impl PortFlow {
    /// Creates a `PortFlow` from an unconstrained rate and inlet temperature.
    ///
    /// # Errors
    ///
    /// - [`StratifiedTankError::NegativePortFlowRate`] if `rate` is negative,
    ///   non-finite, or NaN.
    /// - [`StratifiedTankError::InvalidInletTemperature`] if
    ///   `inlet_temperature` is non-finite or NaN.
    pub fn new(
        rate: VolumeRate,
        inlet_temperature: ThermodynamicTemperature,
    ) -> Result<Self, StratifiedTankError> {
        // Check via raw f64: rejects negatives directly, and `!is_finite()`
        // catches NaN and infinity without negating a partial-order comparison.
        if rate.value < 0.0 || !rate.is_finite() {
            return Err(StratifiedTankError::NegativePortFlowRate(rate));
        }
        if !inlet_temperature.is_finite() {
            return Err(StratifiedTankError::InvalidInletTemperature(
                inlet_temperature,
            ));
        }
        Ok(Self {
            rate,
            inlet_temperature,
        })
    }

    /// Returns the volumetric flow rate shared by the inlet and outlet.
    #[must_use]
    pub fn rate(&self) -> VolumeRate {
        self.rate
    }

    /// Extracts the flow rate, consuming `self`.
    #[must_use]
    pub fn into_rate(self) -> VolumeRate {
        self.rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::{ThermodynamicTemperature, VolumeRate},
        thermodynamic_temperature::kelvin,
        volume_rate::cubic_meter_per_second,
    };

    fn rate(v: f64) -> VolumeRate {
        VolumeRate::new::<cubic_meter_per_second>(v)
    }

    fn temp(k: f64) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(k)
    }

    #[test]
    fn zero_rate_ok() {
        assert!(PortFlow::new(rate(0.0), temp(300.0)).is_ok());
    }

    #[test]
    fn positive_rate_ok() {
        assert!(PortFlow::new(rate(1.0), temp(300.0)).is_ok());
    }

    #[test]
    fn negative_rate_errors() {
        assert!(matches!(
            PortFlow::new(rate(-0.001), temp(300.0)),
            Err(StratifiedTankError::NegativePortFlowRate(_))
        ));
    }

    #[test]
    fn nan_rate_errors() {
        assert!(matches!(
            PortFlow::new(rate(f64::NAN), temp(300.0)),
            Err(StratifiedTankError::NegativePortFlowRate(_))
        ));
    }

    #[test]
    fn infinite_rate_errors() {
        assert!(matches!(
            PortFlow::new(rate(f64::INFINITY), temp(300.0)),
            Err(StratifiedTankError::NegativePortFlowRate(_))
        ));
    }

    #[test]
    fn nan_temperature_errors() {
        assert!(matches!(
            PortFlow::new(rate(1.0), temp(f64::NAN)),
            Err(StratifiedTankError::InvalidInletTemperature(_))
        ));
    }

    #[test]
    fn rate_accessor_returns_stored_value() {
        let pf = PortFlow::new(rate(2.5), temp(300.0)).unwrap();
        assert_eq!(pf.rate(), rate(2.5));
        assert_eq!(pf.into_rate(), rate(2.5));
    }
}
