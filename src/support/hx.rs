//! Heat exchanger analysis toolkit.
//!
//! This module provides general-purpose utilities for heat exchanger analysis,
//! including effectiveness-NTU relationships for various flow arrangements.
//!
//! # Overview
//!
//! Heat exchangers transfer thermal energy between two fluid streams. The
//! effectiveness-NTU method relates exchanger performance to its thermal size
//! (NTU = UA / `C_min`) and the capacity ratio of the streams.
//!
//! This toolkit provides:
//!
//! - **Core types**: [`CapacitanceRate`], [`Effectiveness`], [`Ntu`], [`HeatFlow`]
//! - **Stream modeling**: [`StreamInlet`], [`Stream`]
//! - **Flow arrangements**: [`CounterFlow`], [`ParallelFlow`], [`CrossFlow`], [`ShellAndTube`]
//! - **Analysis functions**: [`functional::known_conductance_and_inlets`],
//!   [`functional::known_conditions_and_inlets`]
//!
//! # Example
//!
//! ```
//! use twine_models::support::constraint::ConstraintResult;
//! use twine_models::support::hx::{
//!     arrangement::CounterFlow,
//!     functional::known_conductance_and_inlets,
//!     CapacitanceRate, StreamInlet,
//! };
//! use uom::si::{
//!     f64::{ThermalConductance, ThermodynamicTemperature},
//!     thermal_conductance::kilowatt_per_kelvin,
//!     thermodynamic_temperature::degree_celsius,
//! };
//!
//! fn main() -> ConstraintResult<()> {
//!     let result = known_conductance_and_inlets(
//!         &CounterFlow,
//!         ThermalConductance::new::<kilowatt_per_kelvin>(3.0),
//!         [
//!             StreamInlet::new(
//!                 CapacitanceRate::new::<kilowatt_per_kelvin>(3.0)?,
//!                 ThermodynamicTemperature::new::<degree_celsius>(50.0),
//!             ),
//!             StreamInlet::new(
//!                 CapacitanceRate::new::<kilowatt_per_kelvin>(6.0)?,
//!                 ThermodynamicTemperature::new::<degree_celsius>(80.0),
//!             ),
//!         ],
//!     )?;
//!
//!     // Access effectiveness and resolved stream states
//!     let _effectiveness = result.effectiveness;
//!     let [cold_stream, hot_stream] = result.streams;
//!     let _cold_outlet = cold_stream.outlet_temperature;
//!     let _hot_outlet = hot_stream.outlet_temperature;
//!
//!     Ok(())
//! }
//! ```

pub mod arrangement;
mod capacitance_rate;
mod capacity_ratio;
mod effectiveness_ntu;
mod flow;
pub mod functional;
mod stream;

pub use arrangement::{CounterFlow, CrossFlow, Mixed, ParallelFlow, ShellAndTube, Unmixed};
pub use capacitance_rate::CapacitanceRate;
pub use capacity_ratio::CapacityRatio;
pub use effectiveness_ntu::{Effectiveness, EffectivenessRelation, Ntu, NtuRelation};
pub use flow::HeatFlow;
pub use stream::{Stream, StreamInlet};
