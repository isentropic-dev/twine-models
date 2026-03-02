use uom::si::f64::{MassDensity, SpecificHeatCapacity, ThermalConductivity};

/// Constant fluid properties used by the stratified tank.
///
/// The tank assumes an incompressible fluid whose properties do not vary with
/// temperature or pressure.  These values are specified at construction and
/// remain fixed for the lifetime of the tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fluid {
    /// Mass density of the fluid.
    pub density: MassDensity,

    /// Specific heat capacity at constant pressure.
    pub specific_heat: SpecificHeatCapacity,

    /// Thermal conductivity, used to compute node-to-node conduction.
    pub thermal_conductivity: ThermalConductivity,
}
