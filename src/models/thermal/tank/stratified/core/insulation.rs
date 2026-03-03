use uom::si::f64::HeatTransfer;

/// Options for specifying tank insulation.
///
/// The insulation setting determines what heat transfer coefficient (thermal transmittance)
/// is applied between the tank fluid and the surrounding environment on all exposed surfaces.
///
/// * `Adiabatic` — no heat transfer.
/// * `Conductive { bottom, side, top }` — thermal transmittance (U-value) for each face.
///
/// The U-values are specified in W/(m²·K) and are multiplied by the surface area of each
/// face during tank construction to compute the overall thermal conductance (UA in W/K).
///
/// Constructors such as [`Insulation::uniform`] and [`Insulation::conductive`]
/// make it easy to build the desired value.

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Insulation {
    /// The tank is perfectly insulated — no heat transfer to the environment.
    Adiabatic,

    /// Conductive heat loss to the environment.
    ///
    /// Each face may have its own U-value (thermal transmittance).
    /// Values are thermal transmittance in e.g. W/(m²·K).
    Conductive {
        /// Thermal transmittance (U-value) of the bottom face.
        bottom: HeatTransfer,
        /// Thermal transmittance (U-value) of the side face.
        side: HeatTransfer,
        /// Thermal transmittance (U-value) of the top face.
        top: HeatTransfer,
    },
}

impl Insulation {
    /// Construct an adiabatic setting (same as `Insulation::Adiabatic`).
    pub fn adiabatic() -> Self {
        Insulation::Adiabatic
    }

    /// Convenience for a uniform thermal transmittance on every face.
    ///
    /// # Arguments
    ///
    /// * `u` - Thermal transmittance (U-value) in W/(m²·K)
    pub fn uniform(u: HeatTransfer) -> Self {
        Insulation::Conductive {
            bottom: u,
            side: u,
            top: u,
        }
    }

    /// Full constructor when faces differ.
    ///
    /// # Arguments
    ///
    /// * `bottom` - Thermal transmittance (U-value) of the bottom face in W/(m²·K)
    /// * `side` - Thermal transmittance (U-value) of the side face in W/(m²·K)
    /// * `top` - Thermal transmittance (U-value) of the top face in W/(m²·K)
    pub fn conductive(bottom: HeatTransfer, side: HeatTransfer, top: HeatTransfer) -> Self {
        Insulation::Conductive { bottom, side, top }
    }
}
