use uom::{ConstZero, si::f64::Length};

/// Vertical placement for ports and auxiliary heat sources within the tank.
///
/// A location can be:
///
/// - [`Location::point_in_node`]: a discrete node by index
/// - [`Location::point_abs`] / [`Location::point_rel`]: a point at a specific height
/// - [`Location::span_abs`] / [`Location::span_rel`]: a symmetric span around a center height
///
/// Spans distribute weight across nodes proportional to the geometric overlap
/// with each node.  A point that lies exactly on an internal node boundary
/// maps to the lower node.
///
/// Constructors do not validate.  Validation occurs when locations are mapped
/// to node weights during tank construction; invalid locations are reported via
/// [`super::super::StratifiedTankError`].
#[derive(Debug, Clone, Copy)]
pub enum Location {
    /// A discrete node identified by its index (0 = bottom).
    Node(usize),

    /// A point at an absolute or relative vertical position.
    Point(Position),

    /// A symmetric span centered at an absolute or relative vertical position.
    Span(Position, Length),
}

/// Vertical reference for specifying a position in the tank.
#[derive(Debug, Clone, Copy)]
pub enum Position {
    /// A fraction of the total tank height, in `[0, 1]` (0 = bottom, 1 = top).
    Relative(f64),

    /// A physical distance from the bottom of the tank.
    Absolute(Length),
}

/// Port pair placement: the inlet location and the outlet location.
#[derive(Debug, Clone, Copy)]
pub struct PortLocation {
    /// Where fluid enters the tank.
    pub inlet: Location,

    /// Where fluid leaves the tank.
    pub outlet: Location,
}

impl Location {
    /// Point inside the node at the given index (0 = bottom).
    #[must_use]
    pub fn point_in_node(index: usize) -> Self {
        Self::Node(index)
    }

    /// Point at an absolute height.
    #[must_use]
    pub fn point_abs(z: Length) -> Self {
        Self::Point(Position::Absolute(z))
    }

    /// Point at a relative height (`0.0` = bottom, `1.0` = top).
    #[must_use]
    pub fn point_rel(frac: f64) -> Self {
        Self::Point(Position::Relative(frac))
    }

    /// Span centered at an absolute height.
    #[must_use]
    pub fn span_abs(center: Length, span: Length) -> Self {
        Self::Span(Position::Absolute(center), span)
    }

    /// Span centered at a relative height.
    #[must_use]
    pub fn span_rel(center_frac: f64, span: Length) -> Self {
        Self::Span(Position::Relative(center_frac), span)
    }

    /// Convenience: a point at the very bottom of the tank.
    #[must_use]
    pub fn tank_bottom() -> Self {
        Self::point_rel(0.0)
    }

    /// Convenience: a point at the very top of the tank.
    #[must_use]
    pub fn tank_top() -> Self {
        Self::point_rel(1.0)
    }

    /// Converts this location into a per-node weight distribution.
    ///
    /// Weights are non-negative and sum to 1.0 for all valid locations.
    ///
    /// # Errors
    ///
    /// Returns a human-readable error string if the location is invalid for
    /// the given node heights (e.g. out of bounds, invalid span).
    pub(super) fn into_weights<const N: usize>(
        self,
        heights: &[Length; N],
    ) -> Result<[f64; N], String> {
        if N == 0 {
            return Err("must have at least one node".into());
        }

        // Cumulative tops of each node (bottom of node 0 is 0).
        let node_tops: [Length; N] = {
            let mut acc = Length::ZERO;
            heights.map(|h| {
                acc += h;
                acc
            })
        };
        let total_height = node_tops[N - 1];

        let mut weights = [0.0_f64; N];

        match self {
            Location::Node(index) => {
                if index >= N {
                    return Err(format!("node index must be in 0..{N}, got {index}"));
                }
                weights[index] = 1.0;
            }

            Location::Point(position) => {
                let z = position.to_abs(total_height)?;
                // Reject negative, infinite, NaN, and out-of-bounds values.
                // Use `.value` to check the raw SI f64, avoiding negated partial-order
                // comparisons (which have surprising behaviour for NaN).
                if z.value < 0.0 || !z.value.is_finite() || z > total_height {
                    return Err(format!(
                        "location out of bounds: {z:?} not within [0, {total_height:?}]"
                    ));
                }

                // A point on an internal boundary maps to the lower node.
                let index = node_tops
                    .partition_point(|&node_top| z > node_top)
                    .min(N - 1);

                weights[index] = 1.0;
            }

            Location::Span(position, span) => {
                let center = position.to_abs(total_height)?;

                // Reject non-positive, infinite, and NaN spans.
                if span.value <= 0.0 || !span.value.is_finite() {
                    return Err(format!("span must be > 0, got {span:?}"));
                }

                let (z0, z1) = (center - span * 0.5, center + span * 0.5);

                // Reject out-of-bounds spans. If center and span are finite
                // (validated above), z0 and z1 are also finite.
                if z0.value < 0.0 || z1.value > total_height.value {
                    return Err(format!(
                        "location out of bounds: [{z0:?}, {z1:?}] not within [0, {total_height:?}]"
                    ));
                }

                let mut start = Length::ZERO;
                for i in 0..N {
                    let end = node_tops[i];
                    let lo = if z0 > start { z0 } else { start };
                    let hi = if z1 < end { z1 } else { end };
                    if hi > lo {
                        weights[i] = ((hi - lo) / span).into();
                    }
                    start = end;
                }
            }
        }

        Ok(weights)
    }
}

impl Position {
    fn to_abs(self, total: Length) -> Result<Length, String> {
        match self {
            Position::Absolute(z) => Ok(z),
            Position::Relative(frac) => {
                if (0.0..=1.0).contains(&frac) {
                    Ok(total * frac)
                } else {
                    Err(format!("relative fraction must be in [0, 1], got {frac}"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::length::meter;

    fn m(v: f64) -> Length {
        Length::new::<meter>(v)
    }

    #[test]
    fn point_abs_selects_containing_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_abs(m(1.2));
        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();
        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 1.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_rel_bottom() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let [w0, w1, w2] = Location::tank_bottom().into_weights(&heights).unwrap();
        assert_relative_eq!(w0, 1.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_rel_top() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let [w0, w1, w2] = Location::tank_top().into_weights(&heights).unwrap();
        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 1.0);
    }

    #[test]
    fn point_on_internal_boundary_maps_to_lower_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let [w0, w1, w2] = Location::point_abs(m(2.0)).into_weights(&heights).unwrap();
        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 1.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_rel_out_of_range_errors() {
        let heights = [m(1.0), m(1.0)];
        assert!(Location::point_rel(-0.01).into_weights(&heights).is_err());
        assert!(Location::point_rel(1.01).into_weights(&heights).is_err());
    }

    #[test]
    fn point_abs_out_of_bounds_errors() {
        let heights = [m(1.0), m(1.0)];
        assert!(Location::point_abs(m(2.1)).into_weights(&heights).is_err());
    }

    #[test]
    fn point_in_node_zero() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let [w0, w1, w2] = Location::point_in_node(0).into_weights(&heights).unwrap();
        assert_relative_eq!(w0, 1.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_in_node_middle() {
        let heights = [m(1.0), m(1.0), m(1.0), m(1.0), m(1.0)];
        let [w0, w1, w2, w3, w4] = Location::point_in_node(2).into_weights(&heights).unwrap();
        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 1.0);
        assert_relative_eq!(w3, 0.0);
        assert_relative_eq!(w4, 0.0);
    }

    #[test]
    fn point_in_node_out_of_range_errors() {
        let heights = [m(1.0), m(1.0)];
        let err = Location::point_in_node(2)
            .into_weights(&heights)
            .unwrap_err();
        assert!(err.contains("node index must be in 0..2"));
    }

    #[test]
    fn span_within_single_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let [w0, w1, w2] = Location::span_abs(m(0.5), m(0.5))
            .into_weights(&heights)
            .unwrap();
        assert_relative_eq!(w0, 1.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn span_crosses_two_nodes() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        // Span [0.65, 1.15]: 0.35 m in node 0 (70%), 0.15 m in node 1 (30%).
        let [w0, w1, w2] = Location::span_abs(m(0.9), m(0.5))
            .into_weights(&heights)
            .unwrap();
        assert_relative_eq!(w0, 0.70);
        assert_relative_eq!(w1, 0.30);
        assert_relative_eq!(w2, 0.00);
    }

    #[test]
    fn span_over_three_of_five_nodes() {
        let heights = [m(1.0), m(1.0), m(1.0), m(1.0), m(1.0)];
        // Center 0.5 (= 2.5 m), span 2 m → [1.5, 3.5].
        let [w0, w1, w2, w3, w4] = Location::span_rel(0.5, m(2.0))
            .into_weights(&heights)
            .unwrap();
        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 0.25);
        assert_relative_eq!(w2, 0.50);
        assert_relative_eq!(w3, 0.25);
        assert_relative_eq!(w4, 0.0);
    }

    #[test]
    fn span_full_tank_proportional_to_node_heights() {
        let heights = [m(1.0), m(2.0), m(3.0)];
        let [w0, w1, w2] = Location::span_rel(0.5, m(6.0))
            .into_weights(&heights)
            .unwrap();
        assert_relative_eq!(w0, 1.0 / 6.0);
        assert_relative_eq!(w1, 2.0 / 6.0);
        assert_relative_eq!(w2, 3.0 / 6.0);
    }

    #[test]
    fn tiny_span_on_boundary_splits_evenly() {
        let heights = [m(0.1), m(0.2)];
        let [w0, w1] = Location::span_abs(m(0.1), m(1e-6))
            .into_weights(&heights)
            .unwrap();
        assert_relative_eq!(w0, 0.5, epsilon = 1e-12);
        assert_relative_eq!(w1, 0.5, epsilon = 1e-12);
    }

    #[test]
    fn span_out_of_bounds_errors() {
        let heights = [m(1.0), m(1.0)];
        let err = Location::span_abs(m(1.5), m(1.5))
            .into_weights(&heights)
            .unwrap_err();
        assert!(err.contains("out of bounds"));
    }

    #[test]
    fn zero_span_errors() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let err = Location::span_abs(m(0.5), m(0.0))
            .into_weights(&heights)
            .unwrap_err();
        assert!(err.contains("span must be > 0"));
    }

    #[test]
    fn negative_span_errors() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let err = Location::span_abs(m(0.5), m(-0.1))
            .into_weights(&heights)
            .unwrap_err();
        assert!(err.contains("span must be > 0"));
    }

    #[test]
    fn nan_span_errors() {
        let heights = [m(1.0), m(1.0)];
        let err = Location::span_abs(m(0.5), m(f64::NAN))
            .into_weights(&heights)
            .unwrap_err();
        assert!(err.contains("span must be > 0"));
    }
}
