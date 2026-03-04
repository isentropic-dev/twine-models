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

/// Deprecated: use [`RecuperatorGivenUa`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUa")]
pub type Recuperator<Fluid, Thermo> = RecuperatorGivenUa<Fluid, Thermo>;

/// Deprecated: use [`RecuperatorGivenUaConfig`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaConfig")]
pub type RecuperatorConfig = RecuperatorGivenUaConfig;

/// Deprecated: use [`RecuperatorGivenUaInput`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaInput")]
pub type RecuperatorInput<Fluid> = RecuperatorGivenUaInput<Fluid>;

/// Deprecated: use [`RecuperatorGivenUaOutput`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaOutput")]
pub type RecuperatorOutput<Fluid> = RecuperatorGivenUaOutput<Fluid>;

/// Deprecated: use [`RecuperatorGivenUaError`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaError")]
pub type RecuperatorError = RecuperatorGivenUaError;
