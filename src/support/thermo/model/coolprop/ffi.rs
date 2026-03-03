//! Raw FFI bindings to the CoolProp C API.
//!
//! This module declares the four `AbstractState_*` functions used from
//! `CoolPropLib.h` and typed constants for input pairs and output parameters.
//! Everything here is `unsafe` — use the wrapper layer above.

use std::os::raw::{c_char, c_double, c_long};

// ── Extern declarations ───────────────────────────────────────────────────────

unsafe extern "C" {
    /// Create a new `AbstractState` and return its integer handle.
    ///
    /// Returns `-1` on failure; check `errcode` and `message_buffer`.
    pub fn AbstractState_factory(
        backend: *const c_char,
        fluids: *const c_char,
        errcode: *mut c_long,
        message_buffer: *mut c_char,
        buffer_length: c_long,
    ) -> c_long;

    /// Release an `AbstractState` created by [`AbstractState_factory`].
    pub fn AbstractState_free(
        handle: c_long,
        errcode: *mut c_long,
        message_buffer: *mut c_char,
        buffer_length: c_long,
    );

    /// Update the thermodynamic state.
    pub fn AbstractState_update(
        handle: c_long,
        input_pair: c_long,
        value1: c_double,
        value2: c_double,
        errcode: *mut c_long,
        message_buffer: *mut c_char,
        buffer_length: c_long,
    );

    /// Query a single output parameter from the current state.
    ///
    /// Returns `f64::MAX` (HUGE_VAL equivalent) on failure; check `errcode`.
    pub fn AbstractState_keyed_output(
        handle: c_long,
        param: c_long,
        errcode: *mut c_long,
        message_buffer: *mut c_char,
        buffer_length: c_long,
    ) -> c_double;
}

// ── Input pairs ───────────────────────────────────────────────────────────────

/// Typed input-pair codes for [`AbstractState_update`].
///
/// Values match the `CoolProp::input_pairs` C++ enum in `DataStructures.h`.
/// Only the pairs used in this crate are declared here; extend as needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum InputPair {
    /// Mass density (kg/m³) + temperature (K).
    DMassT = 10,
    /// Pressure (Pa) + temperature (K).
    PT = 9,
    /// Mass enthalpy (J/kg) + pressure (Pa).
    HMassP = 20,
    /// Pressure (Pa) + mass entropy (J/kg/K).
    PSMass = 22,
    /// Mass enthalpy (J/kg) + mass entropy (J/kg/K).
    HMassSMass = 26,
}

// ── Output parameters ─────────────────────────────────────────────────────────

/// Typed output-parameter codes for [`AbstractState_keyed_output`].
///
/// Values match the `CoolProp::parameters` C++ enum in `DataStructures.h`.
/// Only the parameters used in this crate are declared here; extend as needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum OutputParam {
    /// Molar mass (kg/mol) — a trivial (state-independent) property.
    MolarMass = 2,
    /// Temperature (K).
    T = 19,
    /// Pressure (Pa).
    P = 20,
    /// Mass-based density (kg/m³).
    DMass = 39,
    /// Mass-based enthalpy (J/kg).
    HMass = 40,
    /// Mass-based entropy (J/kg/K).
    SMass = 41,
    /// Mass-based constant-pressure specific heat (J/kg/K).
    CpMass = 42,
    /// Mass-based constant-volume specific heat (J/kg/K).
    CvMass = 44,
    /// Mass-based internal energy (J/kg).
    UMass = 45,
}
