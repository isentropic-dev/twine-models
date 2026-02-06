use uom::si::{
    f64::SpecificHeatCapacity,
    specific_heat_capacity::joule_per_kilogram_kelvin,
};

use crate::support::thermo::model::perfect_gas::{PerfectGasFluid, PerfectGasParameters};
use crate::support::units::SpecificGasConstant;

/// Canonical identifier for dry air.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Air;

// TODO: Add TimeIntegrable impl when available in twine_core
// impl TimeIntegrable for Air {
//     type Derivative = ();
//
//     fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
//         self
//     }
// }

impl PerfectGasFluid for Air {
    fn parameters() -> PerfectGasParameters {
        PerfectGasParameters::new(
            SpecificGasConstant::new::<joule_per_kilogram_kelvin>(287.053),
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1005.0),
        )
    }
}
