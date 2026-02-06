use uom::si::{
    f64::{TemperatureInterval, ThermodynamicTemperature},
    temperature_interval::kelvin as delta_kelvin,
    thermodynamic_temperature::kelvin as abs_kelvin,
};

/// Extension trait for computing temperature differences.
///
/// This trait provides a [`minus`](Self::minus) method that subtracts two
/// [`ThermodynamicTemperature`] values (absolute temperatures) and returns a
/// [`TemperatureInterval`] (temperature difference).
///
/// For background on this distinction and why this extension is needed:
/// [#380](https://github.com/iliekturtles/uom/issues/380),
/// [#289](https://github.com/iliekturtles/uom/issues/289),
/// [#403](https://github.com/iliekturtles/uom/issues/403).
///
/// [`TemperatureInterval`]: uom::si::f64::TemperatureInterval
/// [`ThermodynamicTemperature`]: uom::si::f64::ThermodynamicTemperature
pub trait TemperatureDifference {
    /// Returns the temperature difference `self - other`.
    fn minus(self, other: Self) -> TemperatureInterval;
}

impl TemperatureDifference for ThermodynamicTemperature {
    fn minus(self, other: Self) -> TemperatureInterval {
        TemperatureInterval::new::<delta_kelvin>(
            self.get::<abs_kelvin>() - other.get::<abs_kelvin>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::ThermodynamicTemperature,
        temperature_interval::{degree_celsius as delta_celsius, kelvin as delta_kelvin},
        thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin as abs_kelvin},
    };

    #[test]
    fn subtract_temperatures() {
        let t1 = ThermodynamicTemperature::new::<abs_kelvin>(300.0);
        let t2 = ThermodynamicTemperature::new::<abs_kelvin>(310.0);

        // Positive temperature change.
        assert_relative_eq!(t2.minus(t1).get::<delta_kelvin>(), 10.0);

        // Negative temperature change.
        assert_relative_eq!(t1.minus(t2).get::<delta_celsius>(), -10.0);

        // No difference in temperature between 25°C and 77°F.
        let t_in_c = ThermodynamicTemperature::new::<degree_celsius>(25.0);
        let t_in_f = ThermodynamicTemperature::new::<degree_fahrenheit>(77.0);
        assert_relative_eq!(
            t_in_f.minus(t_in_c).get::<delta_celsius>(),
            0.0,
            epsilon = 1e-12
        );
    }
}
