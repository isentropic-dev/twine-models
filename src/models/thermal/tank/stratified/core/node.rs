use uom::si::f64::ThermalConductance;

use super::{InverseHeatCapacity, InverseVolume};

/// Per-node configuration computed once at tank creation.
///
/// All values in this struct are derived from geometry and fluid properties
/// and remain constant for the lifetime of the tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Node<const P: usize, const Q: usize> {
    /// Reciprocal of node volume (1/m³).
    pub(super) inv_volume: InverseVolume,

    /// Reciprocal of node thermal mass, ρ·c·V  (K/J).
    pub(super) inv_heat_capacity: InverseHeatCapacity,

    /// Overall conductance values at the bottom, side, and top faces.
    pub(super) ua: Adjacent<ThermalConductance>,

    /// Fraction of each auxiliary heat source applied to this node.
    pub(super) aux_heat_weights: [f64; Q],

    /// Fraction of each port pair's inlet flow entering this node.
    pub(super) port_inlet_weights: [f64; P],

    /// Fraction of each port pair's outlet flow leaving this node.
    pub(super) port_outlet_weights: [f64; P],
}

/// Values associated with the bottom, side, and top faces of a node.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Adjacent<T> {
    pub(super) bottom: T,
    pub(super) side: T,
    pub(super) top: T,
}
