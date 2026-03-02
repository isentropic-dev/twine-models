use std::f64::consts::PI;

use uom::si::f64::{Area, Length, Volume};

use super::node::Adjacent;

/// Tank geometry options.
///
/// Determines the shape used to compute per-node areas, heights, and volumes
/// during tank construction.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Geometry {
    /// Vertical cylindrical tank with uniform cross-section.
    VerticalCylinder {
        /// Internal diameter of the tank.
        diameter: Length,

        /// Internal height of the tank.
        height: Length,
    },
}

/// Geometric properties of a single discretized node.
///
/// Computed during tank construction from the overall [`Geometry`] and the
/// number of nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct NodeGeometry {
    /// Cross-sectional areas at the bottom, side, and top faces.
    pub(super) area: Adjacent<Area>,

    /// Height of the node.
    pub(super) height: Length,

    /// Volume of the node.
    pub(super) volume: Volume,
}

impl Geometry {
    /// Partitions the tank geometry into `N` equal node geometries.
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid parameter found.
    /// Callers wrap these into [`super::super::StratifiedTankError::InvalidGeometry`].
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn into_node_geometries<const N: usize>(self) -> Result<[NodeGeometry; N], String> {
        if N == 0 {
            return Err(format!("node count must be ≥ 1, got {N}"));
        }

        match self {
            Geometry::VerticalCylinder { diameter, height } => {
                // Reject non-positive, infinite, and NaN values.
                // Use `.value` (raw f64 in SI base units) for the check so that
                // NaN is caught by `!is_finite()` rather than a negated comparison.
                if diameter.value <= 0.0 || !diameter.value.is_finite() {
                    return Err(format!("diameter must be > 0, got {diameter:?}"));
                }
                if height.value <= 0.0 || !height.value.is_finite() {
                    return Err(format!("height must be > 0, got {height:?}"));
                }

                let end_area = PI * diameter * diameter * 0.25;

                #[allow(clippy::cast_precision_loss)]
                let node_height = height / N as f64;

                Ok([NodeGeometry {
                    area: Adjacent {
                        bottom: end_area,
                        side: PI * diameter * node_height,
                        top: end_area,
                    },
                    height: node_height,
                    volume: end_area * node_height,
                }; N])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::{
        ConstZero,
        si::{area::square_meter, length::meter, volume::cubic_meter},
    };

    #[test]
    fn vertical_cylinder_with_two_nodes() {
        let geometry = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(1.0 / PI.sqrt()), // top/bottom area = 0.25 m²
            height: Length::new::<meter>(2.5),
        };

        let [bottom_node, top_node] = geometry.into_node_geometries().unwrap();

        assert_eq!(
            bottom_node, top_node,
            "all nodes in a vertical cylinder have identical geometry"
        );

        assert_relative_eq!(bottom_node.area.bottom.get::<square_meter>(), 0.25);
        assert_relative_eq!(
            bottom_node.area.side.get::<square_meter>(),
            PI / PI.sqrt() * 1.25
        );
        assert_relative_eq!(bottom_node.area.top.get::<square_meter>(), 0.25);
        assert_relative_eq!(bottom_node.height.get::<meter>(), 1.25);
        assert_relative_eq!(bottom_node.volume.get::<cubic_meter>(), 0.3125);
    }

    #[test]
    fn zero_diameter_errors() {
        let g = Geometry::VerticalCylinder {
            diameter: Length::ZERO,
            height: Length::new::<meter>(1.0),
        };
        assert!(g.into_node_geometries::<3>().is_err());
    }

    #[test]
    fn negative_diameter_errors() {
        let g = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(-0.5),
            height: Length::new::<meter>(1.0),
        };
        assert!(g.into_node_geometries::<3>().is_err());
    }

    #[test]
    fn nan_diameter_errors() {
        let g = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(f64::NAN),
            height: Length::new::<meter>(1.0),
        };
        assert!(g.into_node_geometries::<3>().is_err());
    }

    #[test]
    fn zero_height_errors() {
        let g = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(0.5),
            height: Length::ZERO,
        };
        assert!(g.into_node_geometries::<3>().is_err());
    }

    #[test]
    fn nan_height_errors() {
        let g = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(0.5),
            height: Length::new::<meter>(f64::NAN),
        };
        assert!(g.into_node_geometries::<3>().is_err());
    }

    #[test]
    fn zero_node_count_errors() {
        let g = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(0.5),
            height: Length::new::<meter>(1.0),
        };
        // N = 0 is invalid.
        let result: Result<[NodeGeometry; 0], _> = g.into_node_geometries();
        assert!(result.is_err());
    }
}
