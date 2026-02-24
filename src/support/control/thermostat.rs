//! Thermostat controllers for temperature regulation.
//!
//! This module provides setpoint-based thermostat logic with hysteresis
//! (deadband) to prevent rapid cycling between on/off states.

mod setpoint;

pub use setpoint::{SetpointThermostat, SetpointThermostatInput};
