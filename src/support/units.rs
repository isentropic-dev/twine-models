//! Extensions to [`uom`].
//!
//! This crate uses [`uom`] for all physical units (e.g., temperature, pressure, power).
//! This module provides extensions that are useful for modeling but aren't included in [`uom`].
//!
//! ## Temperature differences
//!
//! The [`TemperatureDifference`] trait provides a [`minus`](TemperatureDifference::minus) method
//! for subtracting one absolute temperature from another to get a temperature interval:
//!
//! ```
//! use uom::si::f64::ThermodynamicTemperature;
//! use uom::si::thermodynamic_temperature::kelvin;
//! use twine_models::support::units::TemperatureDifference;
//!
//! let t1 = ThermodynamicTemperature::new::<kelvin>(300.0);
//! let t2 = ThermodynamicTemperature::new::<kelvin>(250.0);
//! let delta_t = t1.minus(t2);
//! // delta_t is a TemperatureInterval, not a ThermodynamicTemperature
//! ```
//!
//! This extension trait is currently needed due to limitations in [`uom`].
//! See [`TemperatureDifference`] for details.

mod quantities;
mod temperature_difference;

pub use quantities::{
    SpecificEnthalpy, SpecificEntropy, SpecificGasConstant, SpecificInternalEnergy,
};
pub use temperature_difference::TemperatureDifference;
