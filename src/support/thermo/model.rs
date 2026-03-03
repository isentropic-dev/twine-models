//! Thermodynamic property models.

pub mod incompressible;
pub mod perfect_gas;

pub(crate) mod ideal_gas_eos;

#[cfg(any(feature = "coolprop-static", feature = "coolprop-dylib"))]
pub mod coolprop;

pub use incompressible::Incompressible;
pub use perfect_gas::PerfectGas;

#[cfg(any(feature = "coolprop-static", feature = "coolprop-dylib"))]
pub use coolprop::CoolProp;
