use std::{array, ops::RangeInclusive};

use uom::si::{
    f64::{ThermodynamicTemperature, Volume},
    temperature_interval, thermodynamic_temperature,
};

/// Restores thermal stability in a stack of fluid nodes through buoyancy mixing.
///
/// Merges any adjacent nodes where a warmer node sits below a cooler one,
/// computing a volume-weighted average temperature for the merged block.
/// For an incompressible fluid, volume-weighting is equivalent to
/// mass-weighting.
///
/// This represents an instantaneous equilibration; it does not simulate the
/// transient dynamics of the mixing process.
pub(super) fn stabilize<const N: usize>(
    temp: &mut [ThermodynamicTemperature; N],
    vol: &[Volume; N],
) {
    // Fast path: nothing to do when already stable.
    if temp.windows(2).all(|w| w[0] <= w[1]) {
        return;
    }

    // Stack-based one-pass merge: push each node onto the stack, merging
    // downward whenever the block below is warmer than the current block.
    let mut stack: [Block; N] = array::from_fn(|_| Block::default());
    let mut stack_len = 0_usize;

    for i in 0..N {
        let mut block = Block::new(i, temp[i], vol[i]);

        while stack_len > 0 && stack[stack_len - 1].temp > block.temp {
            block = merge(&stack[stack_len - 1], &block);
            stack_len -= 1;
        }

        stack[stack_len] = block;
        stack_len += 1;
    }

    for block in &stack[..stack_len] {
        for i in block.range.clone() {
            temp[i] = block.temp;
        }
    }
}

/// A contiguous block of nodes with uniform temperature and combined volume.
#[derive(Debug, Clone)]
struct Block {
    range: RangeInclusive<usize>,
    temp: ThermodynamicTemperature,
    vol: Volume,
}

impl Block {
    fn new(index: usize, temp: ThermodynamicTemperature, vol: Volume) -> Self {
        Self {
            range: index..=index,
            temp,
            vol,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            range: 0..=0,
            temp: ThermodynamicTemperature::default(),
            vol: Volume::default(),
        }
    }
}

/// Merges two adjacent blocks (below then above) into one, volume-weighted.
///
/// The arithmetic `(V_b * T_b + V_a * T_a) / (V_b + V_a)` yields a
/// `TemperatureInterval`; we re-wrap it as an absolute temperature in kelvin.
fn merge(below: &Block, above: &Block) -> Block {
    let total_vol = below.vol + above.vol;
    let t_mix = (below.vol * below.temp + above.vol * above.temp) / total_vol;

    // t_mix is a TemperatureInterval that represents an absolute kelvin value.
    let t_k = t_mix.get::<temperature_interval::kelvin>();
    let mixed_temp = ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(t_k);

    Block {
        range: *below.range.start()..=*above.range.end(),
        temp: mixed_temp,
        vol: total_vol,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{thermodynamic_temperature::degree_celsius, volume::cubic_meter};

    fn ts<const N: usize>(temps_c: [f64; N]) -> [ThermodynamicTemperature; N] {
        temps_c.map(ThermodynamicTemperature::new::<degree_celsius>)
    }

    fn vs<const N: usize>(vols_m3: [f64; N]) -> [Volume; N] {
        vols_m3.map(Volume::new::<cubic_meter>)
    }

    #[test]
    fn no_mixing_needed() {
        let vol = vs([1.0; 3]);
        let mut temp = ts([30.0, 40.0, 50.0]);
        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([30.0, 40.0, 50.0]));
    }

    #[test]
    fn fully_inverted_all_mixed() {
        let vol = vs([1.0; 3]);
        let mut temp = ts([50.0, 40.0, 30.0]);
        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([40.0, 40.0, 40.0]));
    }

    #[test]
    fn partial_inversion_mixed_from_unstable_region() {
        let vol = vs([1.0; 5]);
        let mut temp = ts([20.0, 30.0, 50.0, 40.0, 42.0]);
        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([20.0, 30.0, 44.0, 44.0, 44.0]));
    }

    #[test]
    fn uneven_volumes_weighted_average() {
        let vol = vs([1.0, 4.0, 2.0]);
        let mut temp = ts([2.0, 10.0, 4.0]);
        stabilize(&mut temp, &vol);
        // Nodes 1 and 2 are unstable (10 > 4).
        // Merged temp = (4*10 + 2*4) / (4+2) = 48/6 = 8.
        assert_eq!(temp, ts([2.0, 8.0, 8.0]));
    }

    #[test]
    fn equal_temperatures_stable() {
        let vol = vs([1.0; 4]);
        let mut temp = ts([25.0; 4]);
        let before = temp;
        stabilize(&mut temp, &vol);
        assert_eq!(temp, before);
    }

    #[test]
    fn single_node_stable() {
        let vol = vs([1.0]);
        let mut temp = ts([50.0]);
        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([50.0]));
    }
}
