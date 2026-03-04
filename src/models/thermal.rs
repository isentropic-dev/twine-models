//! Thermal component models.
//!
//! Each submodule covers a category of thermal component.
//! Individual models live inside their category module.
//!
//! ## Available models
//!
//! - **Heat exchangers** ([`hx`]) — [`RecuperatorGivenUa`]: counterflow heat
//!   recovery between two streams of the same working fluid, discretized into
//!   segments for real-fluid accuracy.
//!
//! - **Tanks** ([`tank`]) — [`StratifiedTank`]: vertical thermal storage tank
//!   discretized into fully mixed nodes, with port pairs, auxiliary heat
//!   sources, buoyancy mixing, and conduction.
//!
//! [`RecuperatorGivenUa`]: hx::discretized::RecuperatorGivenUa
//! [`StratifiedTank`]: tank::stratified::StratifiedTank

pub mod hx;
pub mod tank;
