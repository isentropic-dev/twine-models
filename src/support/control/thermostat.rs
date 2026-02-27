//! Thermostat controllers for temperature regulation.
//!
//! This module provides setpoint-based thermostat logic with hysteresis
//! (deadband) to prevent rapid cycling between on/off states.

pub mod setpoint;

pub use setpoint::{Deadband, InvalidDeadband, SetpointThermostatInput, cooling, heating};
