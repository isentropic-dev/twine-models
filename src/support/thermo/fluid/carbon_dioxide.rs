use uom::si::{
    f64::SpecificHeatCapacity,
    specific_heat_capacity::joule_per_kilogram_kelvin,
};

use crate::support::thermo::model::perfect_gas::{PerfectGasFluid, PerfectGasParameters};
use crate::support::units::SpecificGasConstant;

/// Canonical identifier for carbon dioxide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CarbonDioxide;

// impl TimeIntegrable for CarbonDioxide {
//     type Derivative = ();
// 
//     fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
//         self
//     }
// }

impl PerfectGasFluid for CarbonDioxide {
    fn parameters() -> PerfectGasParameters {
        PerfectGasParameters::new(
            SpecificGasConstant::new::<joule_per_kilogram_kelvin>(188.92),
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(844.0),
        )
    }
}

#[cfg(feature = "coolprop")]
impl crate::support::thermo::model::coolprop::CoolPropFluid for CarbonDioxide {
    const BACKEND: &'static str = "HEOS";
    const NAME: &'static str = "CarbonDioxide";
}
