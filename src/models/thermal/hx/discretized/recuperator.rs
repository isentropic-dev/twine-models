use std::{error::Error as StdError, marker::PhantomData};

use thiserror::Error;
use twine_core::Model;
use uom::si::f64::{TemperatureInterval, ThermalConductance};

use crate::{
    models::thermal::hx::discretized::core::{
        DiscretizedHx, DiscretizedHxThermoModel, Effectiveness, GivenUaConfig, GivenUaError,
        GivenUaResults, Inlets, Known, MassFlows, MinDeltaT, PressureDrops,
    },
    support::{hx::arrangement::CounterFlow, thermo::State},
};

/// A single-fluid counterflow heat exchanger model for heat recovery.
///
/// `RecuperatorGivenUa` implements [`Model`] for use in cycle simulations where heat
/// is transferred between two streams of the same working fluid, typically a
/// hot stream leaving a turbine and a cold stream leaving a compressor.
///
/// # How it works
///
/// The heat exchanger is discretized into segments (sub heat exchangers) so
/// that thermodynamic properties can vary along the flow path, capturing
/// real-fluid behavior. The number of segments controls the fidelity of
/// this discretization.
///
/// Given a target overall thermal conductance (UA), the solver iterates on
/// the outlet temperature to match it. The achieved UA and outlet states
/// are returned.
///
/// # Streams
///
/// Streams are labeled **top** and **bottom**, referring to their position
/// in a schematic layout. The top stream flows left to right; the bottom
/// stream flows right to left (counterflow). Either stream can be the hot
/// or cold side depending on operating conditions.
///
/// # Segments
///
/// The `segments` parameter controls how many constant-property sub heat
/// exchangers the flow is divided into. More segments improve accuracy for
/// fluids with properties that vary significantly with temperature, at the
/// cost of additional computation.
///
/// Supported values: 1, 5, 10, 20, 50.
///
/// Use 1 segment for quick estimates or analytical verification (reduces to
/// the classical ε-NTU result). Use 10-20 for typical engineering accuracy.
/// Use 50 for convergence studies.
///
/// # Example
///
/// ```
/// use twine_core::Model;
/// use twine_models::{
///     models::thermal::hx::discretized::{
///         RecuperatorGivenUa, RecuperatorGivenUaConfig, RecuperatorGivenUaInput,
///         Inlets, MassFlows, PressureDrops,
///     },
///     support::thermo::{
///         State,
///         fluid::Air,
///         model::PerfectGas,
///     },
/// };
/// use uom::si::{
///     f64::{MassDensity, MassRate, Pressure, ThermalConductance, ThermodynamicTemperature},
///     mass_density::kilogram_per_cubic_meter,
///     mass_rate::kilogram_per_second,
///     pressure::kilopascal,
///     thermal_conductance::watt_per_kelvin,
///     thermodynamic_temperature::kelvin,
/// };
///
/// // Build a perfect-gas thermo model for air.
/// let thermo = PerfectGas::<Air>::new().unwrap();
///
/// // Construct the recuperator: 10 segments, default solver tolerances.
/// let recuperator = RecuperatorGivenUa::new(&thermo, 10, RecuperatorGivenUaConfig::default()).unwrap();
///
/// // Define operating conditions.
/// let hot_inlet = State::new(
///     ThermodynamicTemperature::new::<kelvin>(600.0),
///     MassDensity::new::<kilogram_per_cubic_meter>(1.0),
///     Air,
/// );
/// let cold_inlet = State::new(
///     ThermodynamicTemperature::new::<kelvin>(400.0),
///     MassDensity::new::<kilogram_per_cubic_meter>(2.0),
///     Air,
/// );
///
/// let result = recuperator.call(&RecuperatorGivenUaInput {
///     inlets: Inlets { top: cold_inlet, bottom: hot_inlet },
///     mass_flows: MassFlows::new_unchecked(
///         MassRate::new::<kilogram_per_second>(1.0),
///         MassRate::new::<kilogram_per_second>(1.0),
///     ),
///     pressure_drops: PressureDrops::zero(),
///     ua: ThermalConductance::new::<watt_per_kelvin>(500.0),
/// }).unwrap();
///
/// // The cold side (top) is heated; the hot side (bottom) is cooled.
/// assert!(result.top_outlet.temperature > cold_inlet.temperature);
/// assert!(result.bottom_outlet.temperature < hot_inlet.temperature);
/// ```
#[derive(Debug, Clone)]
pub struct RecuperatorGivenUa<Fluid, Thermo> {
    thermo: Thermo,
    segments: usize,
    config: RecuperatorGivenUaConfig,
    _fluid: PhantomData<Fluid>,
}

/// Solver configuration for [`RecuperatorGivenUa`].
#[derive(Debug, Clone, Copy)]
pub struct RecuperatorGivenUaConfig {
    /// Relative tolerance on UA (dimensionless).
    ///
    /// Convergence is reached when
    /// `|achieved_ua - target_ua| / target_ua < ua_rel_tol`.
    pub ua_rel_tol: f64,

    /// Absolute tolerance on the temperature search variable.
    pub temp_abs_tol: TemperatureInterval,

    /// Maximum number of solver iterations.
    pub max_iters: usize,
}

impl Default for RecuperatorGivenUaConfig {
    fn default() -> Self {
        Self {
            ua_rel_tol: 1e-6,
            temp_abs_tol: TemperatureInterval::new::<uom::si::temperature_interval::kelvin>(1e-6),
            max_iters: 100,
        }
    }
}

/// Inputs for [`RecuperatorGivenUa`].
#[derive(Debug, Clone)]
pub struct RecuperatorGivenUaInput<Fluid> {
    /// Inlet states for top and bottom streams.
    pub inlets: Inlets<Fluid, Fluid>,

    /// Mass flow rates for top and bottom streams (strictly positive).
    pub mass_flows: MassFlows,

    /// Pressure drops for top and bottom streams (non-negative).
    pub pressure_drops: PressureDrops,

    /// Target overall thermal conductance.
    pub ua: ThermalConductance,
}

/// Outputs from [`RecuperatorGivenUa`].
#[derive(Debug, Clone)]
pub struct RecuperatorGivenUaOutput<Fluid> {
    /// Top stream outlet state.
    pub top_outlet: State<Fluid>,

    /// Bottom stream outlet state.
    pub bottom_outlet: State<Fluid>,

    /// Heat transfer rate.
    pub q_dot: super::HeatTransferRate,

    /// Achieved overall thermal conductance.
    pub ua: ThermalConductance,

    /// Heat exchanger effectiveness.
    pub effectiveness: Effectiveness,

    /// Minimum hot-to-cold temperature difference and its location.
    pub min_delta_t: MinDeltaT,

    /// Number of solver iterations.
    pub iterations: usize,
}

/// Errors from recuperator construction and solving.
#[derive(Debug, Error)]
pub enum RecuperatorGivenUaError {
    /// The segment count is not supported.
    #[error("unsupported segment count {0}; supported values are 1, 5, 10, 20, 50")]
    UnsupportedSegments(usize),

    /// The solver failed to converge.
    #[error("recuperator solver failed to converge: {message}")]
    Convergence {
        /// Human-readable details from the convergence failure.
        message: String,

        /// Iteration count when available.
        iterations: Option<usize>,
    },

    /// The target UA is negative.
    #[error("target UA must be non-negative, got {0:?}")]
    NegativeUa(ThermalConductance),

    /// A thermodynamic model operation failed.
    ///
    /// This failure can be from property evaluation or state construction.
    #[error("thermodynamic model failed: {context}")]
    ThermoModelFailed {
        /// Operation context for the thermodynamic model failure.
        context: String,

        /// Underlying thermodynamic model error.
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl<Fluid, Thermo> RecuperatorGivenUa<Fluid, Thermo> {
    /// Creates a recuperator model.
    ///
    /// # Errors
    ///
    /// Returns [`RecuperatorGivenUaError::UnsupportedSegments`] if `segments` is not in
    /// `{1, 5, 10, 20, 50}`.
    pub fn new(
        thermo: Thermo,
        segments: usize,
        config: RecuperatorGivenUaConfig,
    ) -> Result<Self, RecuperatorGivenUaError> {
        if !matches!(segments, 1 | 5 | 10 | 20 | 50) {
            return Err(RecuperatorGivenUaError::UnsupportedSegments(segments));
        }

        Ok(Self {
            thermo,
            segments,
            config,
            _fluid: PhantomData,
        })
    }

    fn solve<const N: usize>(
        &self,
        input: &RecuperatorGivenUaInput<Fluid>,
    ) -> Result<RecuperatorGivenUaOutput<Fluid>, RecuperatorGivenUaError>
    where
        Fluid: Clone,
        Thermo: DiscretizedHxThermoModel<Fluid>,
    {
        let known = Known {
            inlets: input.inlets.clone(),
            m_dot: input.mass_flows,
            dp: input.pressure_drops,
        };

        let ua_abs_tol = input.ua * self.config.ua_rel_tol.abs();
        let given_ua_config = GivenUaConfig {
            max_iters: self.config.max_iters,
            temp_tol: self.config.temp_abs_tol,
            ua_tol: ua_abs_tol,
        };

        let given_ua_results = DiscretizedHx::<CounterFlow, N>::given_ua_same(
            &known,
            input.ua,
            given_ua_config,
            &self.thermo,
        )
        .map_err(RecuperatorGivenUaError::from)?;

        Ok(Self::to_output(given_ua_results))
    }

    fn to_output<const N: usize>(
        given_ua_results: GivenUaResults<Fluid, Fluid, N>,
    ) -> RecuperatorGivenUaOutput<Fluid>
    where
        Fluid: Clone,
    {
        let results = given_ua_results.results;

        RecuperatorGivenUaOutput {
            top_outlet: results.top[N - 1].clone(),
            bottom_outlet: results.bottom[0].clone(),
            q_dot: results.q_dot,
            ua: results.ua,
            effectiveness: results.effectiveness,
            min_delta_t: results.min_delta_t,
            iterations: given_ua_results.iterations,
        }
    }
}

impl<Fluid, Thermo> Model for RecuperatorGivenUa<Fluid, Thermo>
where
    Fluid: Clone,
    Thermo: DiscretizedHxThermoModel<Fluid>,
{
    type Input = RecuperatorGivenUaInput<Fluid>;
    type Output = RecuperatorGivenUaOutput<Fluid>;
    type Error = RecuperatorGivenUaError;

    fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
        match self.segments {
            1 => self.solve::<2>(input),
            5 => self.solve::<6>(input),
            10 => self.solve::<11>(input),
            20 => self.solve::<21>(input),
            50 => self.solve::<51>(input),
            _ => unreachable!("validated at construction"),
        }
    }
}

impl From<GivenUaError> for RecuperatorGivenUaError {
    fn from(value: GivenUaError) -> Self {
        match value {
            GivenUaError::NegativeUa(ua) => Self::NegativeUa(ua),
            GivenUaError::Solve(error) => Self::ThermoModelFailed {
                context: "discretized heat exchanger solve".to_owned(),
                source: Box::new(error),
            },
            GivenUaError::Bisection(error) => Self::Convergence {
                message: error.to_string(),
                iterations: None,
            },
            GivenUaError::MaxIters { iters, .. } => Self::Convergence {
                message: "iteration limit reached".to_owned(),
                iterations: Some(iters),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use twine_core::Model;
    use uom::si::{
        f64::MassRate, mass_rate::kilogram_per_second, thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::kelvin,
    };

    use crate::models::thermal::hx::discretized::core::{
        Inlets, MassFlows, PressureDrops,
        test_support::{TestFluid, TestThermoModel, state},
    };

    fn thermo() -> TestThermoModel {
        TestThermoModel::new()
    }

    fn mass_flows() -> MassFlows {
        MassFlows::new_unchecked(
            MassRate::new::<kilogram_per_second>(1.0),
            MassRate::new::<kilogram_per_second>(1.0),
        )
    }

    fn input(top: f64, bottom: f64, ua_wpk: f64) -> RecuperatorGivenUaInput<TestFluid> {
        RecuperatorGivenUaInput {
            inlets: Inlets {
                top: state(top),
                bottom: state(bottom),
            },
            mass_flows: mass_flows(),
            pressure_drops: PressureDrops::default(),
            ua: ThermalConductance::new::<watt_per_kelvin>(ua_wpk),
        }
    }

    #[test]
    fn new_accepts_supported_segment_counts() {
        for n in [1, 5, 10, 20, 50] {
            assert!(
                RecuperatorGivenUa::<TestFluid, _>::new(
                    thermo(),
                    n,
                    RecuperatorGivenUaConfig::default()
                )
                .is_ok(),
                "segment count {n} should be accepted",
            );
        }
    }

    #[test]
    fn new_rejects_unsupported_segment_counts() {
        for n in [0, 2, 3, 100] {
            assert!(
                matches!(
                    RecuperatorGivenUa::<TestFluid, _>::new(
                        thermo(),
                        n,
                        RecuperatorGivenUaConfig::default()
                    ),
                    Err(RecuperatorGivenUaError::UnsupportedSegments(_))
                ),
                "segment count {n} should be rejected",
            );
        }
    }

    #[test]
    fn call_hot_cools_and_cold_heats() {
        // Bottom is hot (600 K), top is cold (400 K).
        let inp = input(400.0, 600.0, 500.0);
        let cold_inlet_temp = inp.inlets.top.temperature;
        let hot_inlet_temp = inp.inlets.bottom.temperature;

        let recuperator =
            RecuperatorGivenUa::new(thermo(), 10, RecuperatorGivenUaConfig::default()).unwrap();
        let out = recuperator.call(&inp).unwrap();

        assert!(
            out.top_outlet.temperature > cold_inlet_temp,
            "cold side should be heated"
        );
        assert!(
            out.bottom_outlet.temperature < hot_inlet_temp,
            "hot side should be cooled"
        );
    }

    #[test]
    fn zero_ua_returns_inlets_unchanged() {
        let inp = input(400.0, 600.0, 0.0);

        let recuperator =
            RecuperatorGivenUa::new(thermo(), 10, RecuperatorGivenUaConfig::default()).unwrap();
        let out = recuperator.call(&inp).unwrap();

        assert_relative_eq!(out.top_outlet.temperature.get::<kelvin>(), 400.0);
        assert_relative_eq!(out.bottom_outlet.temperature.get::<kelvin>(), 600.0);
    }

    #[test]
    fn negative_ua_returns_error() {
        let recuperator =
            RecuperatorGivenUa::new(thermo(), 10, RecuperatorGivenUaConfig::default()).unwrap();
        let result = recuperator.call(&input(400.0, 600.0, -1.0));

        assert!(
            matches!(result, Err(RecuperatorGivenUaError::NegativeUa(_))),
            "expected NegativeUa error",
        );
    }

    #[cfg(any(feature = "coolprop-static", feature = "coolprop-dylib"))]
    mod coolprop_tests {
        use super::*;

        use crate::support::thermo::{
            capability::StateFrom, fluid::CarbonDioxide, model::CoolProp,
        };
        use uom::si::{
            f64::{Pressure, ThermodynamicTemperature},
            pressure::{megapascal, pascal},
            thermal_conductance::kilowatt_per_kelvin,
            thermodynamic_temperature::degree_celsius,
        };

        #[test]
        fn co2_five_segments_at_atmospheric_pressure() {
            let thermo = CoolProp::<CarbonDioxide>::new().unwrap();

            let atm = Pressure::new::<pascal>(101_325.0);

            let cold_inlet = thermo
                .state_from((
                    CarbonDioxide,
                    ThermodynamicTemperature::new::<degree_celsius>(25.0),
                    atm,
                ))
                .unwrap();
            let hot_inlet = thermo
                .state_from((
                    CarbonDioxide,
                    ThermodynamicTemperature::new::<degree_celsius>(300.0),
                    atm,
                ))
                .unwrap();

            let recuperator =
                RecuperatorGivenUa::new(&thermo, 5, RecuperatorGivenUaConfig::default()).unwrap();

            let result = recuperator
                .call(&RecuperatorGivenUaInput {
                    inlets: Inlets {
                        top: cold_inlet,
                        bottom: hot_inlet,
                    },
                    mass_flows: MassFlows::new_unchecked(
                        MassRate::new::<kilogram_per_second>(1.0),
                        MassRate::new::<kilogram_per_second>(1.0),
                    ),
                    pressure_drops: PressureDrops::zero(),
                    ua: ThermalConductance::new::<watt_per_kelvin>(500.0),
                })
                .unwrap();

            assert!(
                result.top_outlet.temperature > cold_inlet.temperature,
                "cold side should be heated",
            );
            assert!(
                result.bottom_outlet.temperature < hot_inlet.temperature,
                "hot side should be cooled",
            );
        }

        #[test]
        fn co2_five_segments_at_sco2_power_cycle_conditions() {
            let thermo = CoolProp::<CarbonDioxide>::new().unwrap();

            // Typical sCO2 recuperator conditions:
            // Cold side: compressor outlet ~180°C at ~20 MPa
            // Hot side: turbine outlet ~380°C at ~8 MPa
            let cold_inlet = thermo
                .state_from((
                    CarbonDioxide,
                    ThermodynamicTemperature::new::<degree_celsius>(180.0),
                    Pressure::new::<megapascal>(20.0),
                ))
                .unwrap();
            let hot_inlet = thermo
                .state_from((
                    CarbonDioxide,
                    ThermodynamicTemperature::new::<degree_celsius>(380.0),
                    Pressure::new::<megapascal>(8.0),
                ))
                .unwrap();

            let recuperator =
                RecuperatorGivenUa::new(&thermo, 5, RecuperatorGivenUaConfig::default()).unwrap();

            let result = recuperator
                .call(&RecuperatorGivenUaInput {
                    inlets: Inlets {
                        top: cold_inlet,
                        bottom: hot_inlet,
                    },
                    mass_flows: MassFlows::new_unchecked(
                        MassRate::new::<kilogram_per_second>(1.0),
                        MassRate::new::<kilogram_per_second>(1.0),
                    ),
                    pressure_drops: PressureDrops::zero(),
                    ua: ThermalConductance::new::<watt_per_kelvin>(2000.0),
                })
                .unwrap();

            assert!(
                result.top_outlet.temperature > cold_inlet.temperature,
                "cold side should be heated",
            );
            assert!(
                result.bottom_outlet.temperature < hot_inlet.temperature,
                "hot side should be cooled",
            );
        }

        /// Reproduces the conditions from issue #58: sCO₂ near the critical
        /// point (32 °C / 8 `MPa` compressor inlet) with multiple segments.
        /// Cold side approximates compressor outlet, hot side approximates
        /// turbine outlet for a simple recuperated Brayton cycle.
        #[test]
        fn co2_five_segments_near_critical_point() {
            let thermo = CoolProp::<CarbonDioxide>::new().unwrap();

            // Cold side: compressor outlet ~80°C at ~20 MPa
            // (compressor inlet 32°C / 8 MPa, ~2.5 pressure ratio)
            let cold_inlet = thermo
                .state_from((
                    CarbonDioxide,
                    ThermodynamicTemperature::new::<degree_celsius>(80.0),
                    Pressure::new::<megapascal>(20.0),
                ))
                .unwrap();

            // Hot side: turbine outlet ~400°C at ~8 MPa
            let hot_inlet = thermo
                .state_from((
                    CarbonDioxide,
                    ThermodynamicTemperature::new::<degree_celsius>(400.0),
                    Pressure::new::<megapascal>(8.0),
                ))
                .unwrap();

            let recuperator =
                RecuperatorGivenUa::new(&thermo, 5, RecuperatorGivenUaConfig::default()).unwrap();

            let result = recuperator
                .call(&RecuperatorGivenUaInput {
                    inlets: Inlets {
                        top: cold_inlet,
                        bottom: hot_inlet,
                    },
                    mass_flows: MassFlows::new_unchecked(
                        MassRate::new::<kilogram_per_second>(1.0),
                        MassRate::new::<kilogram_per_second>(1.0),
                    ),
                    pressure_drops: PressureDrops::zero(),
                    ua: ThermalConductance::new::<kilowatt_per_kelvin>(2000.0),
                })
                .unwrap();

            assert!(
                result.top_outlet.temperature > cold_inlet.temperature,
                "cold side should be heated",
            );
            assert!(
                result.bottom_outlet.temperature < hot_inlet.temperature,
                "hot side should be cooled",
            );
        }
    }
}

/// Deprecated: use [`RecuperatorGivenUa`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUa")]
pub type Recuperator<Fluid, Thermo> = RecuperatorGivenUa<Fluid, Thermo>;

/// Deprecated: use [`RecuperatorGivenUaConfig`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaConfig")]
pub type RecuperatorConfig = RecuperatorGivenUaConfig;

/// Deprecated: use [`RecuperatorGivenUaInput`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaInput")]
pub type RecuperatorInput<Fluid> = RecuperatorGivenUaInput<Fluid>;

/// Deprecated: use [`RecuperatorGivenUaOutput`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaOutput")]
pub type RecuperatorOutput<Fluid> = RecuperatorGivenUaOutput<Fluid>;

/// Deprecated: use [`RecuperatorGivenUaError`] instead.
#[deprecated(since = "0.3.0", note = "renamed to RecuperatorGivenUaError")]
pub type RecuperatorError = RecuperatorGivenUaError;
