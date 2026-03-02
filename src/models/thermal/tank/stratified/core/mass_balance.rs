use std::array;

use uom::{ConstZero, si::f64::VolumeRate};

/// Computes upward inter-node flows from port pair flow rates and node weights.
///
/// Returns an array of `N` values.  Entry `i` (for `0 ≤ i < N − 1`) is the
/// net volumetric flow from node `i` to node `i + 1`; positive means upward,
/// negative means downward.  Entry `N − 1` is the residual and should be zero
/// if mass is conserved across all port pairs.
///
/// The residual is checked in debug builds (tolerance 1 × 10⁻¹² m³/s).
///
/// # Parameters
///
/// - `port_flow_rates`: Flow rate for each of the `P` port pairs.
/// - `inlet_weights`: For each node, the fraction of each port's inflow entering it.
/// - `outlet_weights`: For each node, the fraction of each port's outflow leaving it.
pub(super) fn compute_upward_flows<const N: usize, const P: usize>(
    port_flow_rates: &[VolumeRate; P],
    inlet_weights: &[[f64; P]; N],
    outlet_weights: &[[f64; P]; N],
) -> [VolumeRate; N] {
    let mut flow_up = VolumeRate::ZERO;

    let upward_flows: [VolumeRate; N] = array::from_fn(|i| {
        // Net inflow to node i from all port pairs:
        // Σ_k[ v_dot[k] * (w_in[i][k] - w_out[i][k]) ]
        let net_port_inflow: VolumeRate = port_flow_rates
            .iter()
            .zip(inlet_weights[i].iter().zip(outlet_weights[i].iter()))
            .map(|(&v_dot, (&wi, &wo))| v_dot * (wi - wo))
            .sum();

        // Cumulative upward flow enforces dV/dt = 0 at each node.
        flow_up += net_port_inflow;
        flow_up
    });

    #[cfg(debug_assertions)]
    {
        use uom::si::volume_rate::cubic_meter_per_second;
        let residual = upward_flows[N - 1].get::<cubic_meter_per_second>();
        assert!(
            residual.abs() < 1e-12,
            "mass is not conserved; residual at top boundary = {residual} m³/s",
        );
    }

    upward_flows
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::volume_rate::cubic_meter_per_second;

    fn rate(v: f64) -> VolumeRate {
        VolumeRate::new::<cubic_meter_per_second>(v)
    }

    fn get(v: VolumeRate) -> f64 {
        v.get::<cubic_meter_per_second>()
    }

    #[test]
    fn single_port_bottom_in_top_out() {
        let port_flow_rates = [rate(1.0)];
        let inlet_weights = [[1.0], [0.0], [0.0]];
        let outlet_weights = [[0.0], [0.0], [1.0]];

        let flow_up = compute_upward_flows(&port_flow_rates, &inlet_weights, &outlet_weights);

        assert_relative_eq!(get(flow_up[0]), 1.0);
        assert_relative_eq!(get(flow_up[1]), 1.0);
        assert_relative_eq!(get(flow_up[2]), 0.0); // residual
    }

    #[test]
    fn inlet_and_outlet_same_node_no_vertical_flow() {
        let port_flow_rates = [rate(0.8)];
        let inlet_weights = [[0.0], [1.0], [0.0]];
        let outlet_weights = [[0.0], [1.0], [0.0]];

        let flow_up = compute_upward_flows(&port_flow_rates, &inlet_weights, &outlet_weights);

        assert_relative_eq!(get(flow_up[0]), 0.0);
        assert_relative_eq!(get(flow_up[1]), 0.0);
        assert_relative_eq!(get(flow_up[2]), 0.0);
    }

    #[test]
    fn two_ports_mixed_distribution() {
        let port_flow_rates = [rate(0.3), rate(0.5)];
        let inlet_weights = [[1.0, 0.0], [0.0, 0.6], [0.0, 0.4]];
        let outlet_weights = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0]];

        // Node 0: 0.3 in - 0.5 out = -0.2
        // Node 1: 0.3 in - 0.0 out =  0.3 (cumulative = -0.2 + 0.3 = 0.1)
        // Node 2: 0.2 in - 0.3 out = -0.1 (residual = 0)
        let flow_up = compute_upward_flows(&port_flow_rates, &inlet_weights, &outlet_weights);

        assert_relative_eq!(get(flow_up[0]), -0.2);
        assert_relative_eq!(get(flow_up[1]), 0.1);
        assert_relative_eq!(get(flow_up[2]), 0.0);
    }

    #[test]
    fn zero_flow_rate_no_vertical_flow() {
        let port_flow_rates = [rate(0.0)];
        let inlet_weights = [[1.0], [0.0], [0.0]];
        let outlet_weights = [[0.0], [0.0], [1.0]];

        let flow_up = compute_upward_flows(&port_flow_rates, &inlet_weights, &outlet_weights);

        for &f in &flow_up {
            assert_relative_eq!(get(f), 0.0);
        }
    }
}
