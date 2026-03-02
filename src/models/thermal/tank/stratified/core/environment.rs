use uom::si::f64::ThermodynamicTemperature;

/// Ambient temperatures surrounding the tank.
///
/// Used to compute conductive heat exchange at the bottom, side, and top
/// surfaces when insulation allows heat transfer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    /// Temperature below the tank (applied to the bottom face of node 0).
    pub bottom: ThermodynamicTemperature,

    /// Temperature at the sides of the tank (applied to the side face of every node).
    pub side: ThermodynamicTemperature,

    /// Temperature above the tank (applied to the top face of the last node).
    pub top: ThermodynamicTemperature,
}
