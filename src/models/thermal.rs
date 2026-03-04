//! Thermal component models.
//!
//! Each submodule covers a category of thermal component.
//! Individual models live inside their category module.
//!
//! ## Available models
//!
//! - **Heat exchangers** ([`hx`]) — counterflow heat recovery between two
//!   streams of the same working fluid, discretized into segments for
//!   real-fluid accuracy:
//!   - [`RecuperatorGivenUa`]: given a target UA, find outlet states
//!     (iterative).
//!   - [`RecuperatorGivenOutlet`]: given an outlet temperature, compute UA
//!     (direct).
//!
//! - **Tanks** ([`tank`]) — [`StratifiedTank`]: vertical thermal storage tank
//!   discretized into fully mixed nodes, with port pairs, auxiliary heat
//!   sources, buoyancy mixing, and conduction.
//!
//! [`RecuperatorGivenUa`]: hx::discretized::RecuperatorGivenUa
//! [`RecuperatorGivenOutlet`]: hx::discretized::RecuperatorGivenOutlet
//! [`StratifiedTank`]: tank::stratified::StratifiedTank

pub mod hx;
pub mod tank;
