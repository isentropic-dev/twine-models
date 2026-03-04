//! Recuperator models for counterflow heat recovery.
//!
//! Two models are provided, distinguished by what is known:
//!
//! - [`RecuperatorGivenUa`]: given a target thermal conductance (UA),
//!   iterates on outlet temperature to find the operating state.
//! - [`RecuperatorGivenOutlet`]: given an outlet temperature, computes
//!   the resulting UA directly (no iteration).

mod given_outlet;
mod given_ua;

pub use given_outlet::{
    OutletTemp, RecuperatorGivenOutlet, RecuperatorGivenOutletError, RecuperatorGivenOutletInput,
    RecuperatorGivenOutletOutput,
};
pub use given_ua::{
    RecuperatorGivenUa, RecuperatorGivenUaConfig, RecuperatorGivenUaError, RecuperatorGivenUaInput,
    RecuperatorGivenUaOutput,
};
