//! Setpoint thermostat with deadband hysteresis.

use std::cmp::Ordering;

use thiserror::Error;
use uom::si::{
    f64::{TemperatureInterval, ThermodynamicTemperature},
    temperature_interval,
};

use crate::support::control::SwitchState;

/// A non-negative temperature interval used as a thermostat deadband.
///
/// Validated at construction — the type carries the guarantee that the
/// deadband is non-negative, so callers don't need to check on every use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deadband(TemperatureInterval);

/// Error returned when constructing a [`Deadband`] with a negative value.
#[derive(Debug, Error)]
#[error("deadband must be non-negative, got {value:?}")]
pub struct InvalidDeadband {
    pub value: TemperatureInterval,
}

impl Deadband {
    /// Constructs a `Deadband` if `value` is non-negative.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeadband`] if `value` is negative or NaN.
    pub fn new(value: TemperatureInterval) -> Result<Self, InvalidDeadband> {
        let zero = TemperatureInterval::new::<temperature_interval::kelvin>(0.0);
        match value.partial_cmp(&zero) {
            Some(Ordering::Greater | Ordering::Equal) => Ok(Self(value)),
            Some(Ordering::Less) | None => Err(InvalidDeadband { value }),
        }
    }

    /// Returns the underlying temperature interval.
    #[must_use]
    pub fn value(self) -> TemperatureInterval {
        self.0
    }
}

/// Controls heating to maintain temperature above a setpoint.
///
/// Turns the heating system on when temperature drops to `setpoint - deadband`
/// or below, and off when temperature reaches the setpoint or higher.
///
/// # Examples
///
/// ```
/// use twine_models::support::control::{
///     SwitchState,
///     thermostat::setpoint::{Deadband, SetpointThermostatInput, heating},
/// };
/// use uom::si::{
///     f64::{TemperatureInterval, ThermodynamicTemperature},
///     temperature_interval::degree_celsius as delta_celsius,
///     thermodynamic_temperature::degree_celsius,
/// };
///
/// let input = SetpointThermostatInput {
///     state: SwitchState::Off,
///     temperature: ThermodynamicTemperature::new::<degree_celsius>(18.0),
///     setpoint: ThermodynamicTemperature::new::<degree_celsius>(20.0),
///     deadband: Deadband::new(TemperatureInterval::new::<delta_celsius>(1.0)).unwrap(),
/// };
///
/// // Temperature is at 18°C, which is at or below 19°C (setpoint - deadband).
/// assert_eq!(heating(input), SwitchState::On);
/// ```
#[must_use]
pub fn heating(input: SetpointThermostatInput) -> SwitchState {
    let SetpointThermostatInput {
        state,
        temperature,
        setpoint,
        deadband,
    } = input;

    match state {
        SwitchState::Off => {
            if temperature <= setpoint - deadband.value() {
                SwitchState::On
            } else {
                SwitchState::Off
            }
        }
        SwitchState::On => {
            if temperature >= setpoint {
                SwitchState::Off
            } else {
                SwitchState::On
            }
        }
    }
}

/// Controls cooling to maintain temperature below a setpoint.
///
/// Turns the cooling system on when temperature rises to `setpoint + deadband`
/// or above, and off when temperature reaches the setpoint or lower.
///
/// # Examples
///
/// ```
/// use twine_models::support::control::{
///     SwitchState,
///     thermostat::setpoint::{Deadband, SetpointThermostatInput, cooling},
/// };
/// use uom::si::{
///     f64::{TemperatureInterval, ThermodynamicTemperature},
///     temperature_interval::degree_celsius as delta_celsius,
///     thermodynamic_temperature::degree_celsius,
/// };
///
/// let input = SetpointThermostatInput {
///     state: SwitchState::Off,
///     temperature: ThermodynamicTemperature::new::<degree_celsius>(18.0),
///     setpoint: ThermodynamicTemperature::new::<degree_celsius>(20.0),
///     deadband: Deadband::new(TemperatureInterval::new::<delta_celsius>(1.0)).unwrap(),
/// };
///
/// // Temperature is at 18°C, which is below 21°C (setpoint + deadband).
/// assert_eq!(cooling(input), SwitchState::Off);
/// ```
#[must_use]
pub fn cooling(input: SetpointThermostatInput) -> SwitchState {
    let SetpointThermostatInput {
        state,
        temperature,
        setpoint,
        deadband,
    } = input;

    match state {
        SwitchState::Off => {
            if temperature >= setpoint + deadband.value() {
                SwitchState::On
            } else {
                SwitchState::Off
            }
        }
        SwitchState::On => {
            if temperature <= setpoint {
                SwitchState::Off
            } else {
                SwitchState::On
            }
        }
    }
}

/// Input to the [`heating`] and [`cooling`] thermostat functions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetpointThermostatInput {
    /// The current on/off state of the controlled system (e.g., heater or cooler).
    pub state: SwitchState,

    /// The current measured temperature.
    pub temperature: ThermodynamicTemperature,

    /// The desired setpoint temperature to maintain.
    pub setpoint: ThermodynamicTemperature,

    /// The deadband around the setpoint to avoid rapid cycling.
    pub deadband: Deadband,
}

impl SetpointThermostatInput {
    /// Returns `self` with the given state, keeping other fields unchanged.
    #[must_use]
    pub fn with_state(self, state: SwitchState) -> Self {
        Self { state, ..self }
    }

    /// Returns `self` with the given temperature, keeping other fields unchanged.
    #[must_use]
    pub fn with_temperature(self, temperature: ThermodynamicTemperature) -> Self {
        Self {
            temperature,
            ..self
        }
    }

    /// Returns `self` with the given setpoint, keeping other fields unchanged.
    #[must_use]
    pub fn with_setpoint(self, setpoint: ThermodynamicTemperature) -> Self {
        Self { setpoint, ..self }
    }

    /// Returns `self` with the given deadband, keeping other fields unchanged.
    #[must_use]
    pub fn with_deadband(self, deadband: Deadband) -> Self {
        Self { deadband, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        temperature_interval::degree_celsius as delta_celsius,
        thermodynamic_temperature::degree_celsius,
    };

    /// Setpoint (°C) for all tests.
    const SETPOINT: f64 = 20.0;

    /// Deadband (°C) for all tests.
    const DEADBAND: f64 = 2.0;

    fn test_input(state: SwitchState, temperature: f64) -> SetpointThermostatInput {
        SetpointThermostatInput {
            state,
            temperature: ThermodynamicTemperature::new::<degree_celsius>(temperature),
            setpoint: ThermodynamicTemperature::new::<degree_celsius>(SETPOINT),
            deadband: Deadband::new(TemperatureInterval::new::<delta_celsius>(DEADBAND)).unwrap(),
        }
    }

    mod deadband {
        use super::*;

        #[test]
        fn rejects_negative() {
            let negative = TemperatureInterval::new::<delta_celsius>(-1.0);
            assert!(Deadband::new(negative).is_err());
        }

        #[test]
        fn rejects_nan() {
            let nan = TemperatureInterval::new::<delta_celsius>(f64::NAN);
            assert!(Deadband::new(nan).is_err());
        }

        #[test]
        fn accepts_zero() {
            let zero = TemperatureInterval::new::<delta_celsius>(0.0);
            assert!(Deadband::new(zero).is_ok());
        }

        #[test]
        fn accepts_positive() {
            let positive = TemperatureInterval::new::<delta_celsius>(2.0);
            assert!(Deadband::new(positive).is_ok());
        }
    }

    mod heating {
        use super::*;

        #[test]
        fn turns_on_at_or_below_threshold() {
            let on_threshold = SETPOINT - DEADBAND;

            let input = test_input(SwitchState::Off, on_threshold);
            assert_eq!(super::super::heating(input), SwitchState::On);

            let input = test_input(SwitchState::Off, on_threshold - 0.1);
            assert_eq!(super::super::heating(input), SwitchState::On);
        }

        #[test]
        fn stays_on_below_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT - 0.1);
            assert_eq!(super::super::heating(input), SwitchState::On);
        }

        #[test]
        fn turns_off_at_or_above_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT);
            assert_eq!(super::super::heating(input), SwitchState::Off);

            let input = test_input(SwitchState::On, SETPOINT + 0.1);
            assert_eq!(super::super::heating(input), SwitchState::Off);
        }

        #[test]
        fn stays_off_in_deadband() {
            let on_threshold = SETPOINT - DEADBAND;
            let midpoint = f64::midpoint(SETPOINT, on_threshold);

            let input = test_input(SwitchState::Off, midpoint);
            assert_eq!(super::super::heating(input), SwitchState::Off);
        }
    }

    mod cooling {
        use super::*;

        #[test]
        fn turns_on_at_or_above_threshold() {
            let on_threshold = SETPOINT + DEADBAND;

            let input = test_input(SwitchState::Off, on_threshold);
            assert_eq!(super::super::cooling(input), SwitchState::On);

            let input = test_input(SwitchState::Off, on_threshold + 0.1);
            assert_eq!(super::super::cooling(input), SwitchState::On);
        }

        #[test]
        fn stays_on_above_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT + 0.1);
            assert_eq!(super::super::cooling(input), SwitchState::On);
        }

        #[test]
        fn turns_off_at_or_below_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT);
            assert_eq!(super::super::cooling(input), SwitchState::Off);

            let input = test_input(SwitchState::On, SETPOINT - 0.1);
            assert_eq!(super::super::cooling(input), SwitchState::Off);
        }

        #[test]
        fn stays_off_in_deadband() {
            let on_threshold = SETPOINT + DEADBAND;
            let midpoint = f64::midpoint(SETPOINT, on_threshold);

            let input = test_input(SwitchState::Off, midpoint);
            assert_eq!(super::super::cooling(input), SwitchState::Off);
        }
    }
}
