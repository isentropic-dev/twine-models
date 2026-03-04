use std::{error::Error as StdError, marker::PhantomData};

use thiserror::Error;
use twine_core::Model;
use uom::si::f64::{ThermalConductance, ThermodynamicTemperature};

use crate::{
    models::thermal::hx::discretized::core::{
        DiscretizedHx, DiscretizedHxThermoModel, Effectiveness, Given, HeatTransferRate, Inlets,
        Known, MassFlows, MinDeltaT, PressureDrops, Results, SolveError,
    },
    support::{hx::arrangement::CounterFlow, thermo::State},
};

/// A single-fluid counterflow heat exchanger model for heat recovery,
/// solving for UA given a specified outlet temperature.
///
/// `RecuperatorGivenOutlet` implements [`Model`] for use in cycle solvers
/// where recuperator outlet temperatures are iteration variables.
/// Given an outlet temperature, it computes the resulting thermal
/// conductance (UA) directly — no internal iteration required.
///
/// # When to use this vs [`RecuperatorGivenUa`]
///
/// Use `RecuperatorGivenOutlet` when outlet temperatures are known
/// (e.g., as iteration variables in an external solver) and you need
/// the resulting UA to compare against a target.
/// Use [`RecuperatorGivenUa`] when UA is fixed and you want to find
/// the outlet states.
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
/// exchangers the flow is divided into. More segments improve accuracy
/// for fluids with properties that vary significantly with temperature,
/// at the cost of additional computation. Internally, `segments` maps
/// to a const-generic node count (`N = segments + 1`).
///
/// Supported values: 1, 5, 10, 20, 50. These are a practical subset —
/// additional values can be added with no runtime cost (the tradeoff is
/// compile time and binary size from const-generic monomorphization).
///
/// [`RecuperatorGivenUa`]: super::RecuperatorGivenUa
#[derive(Debug, Clone)]
pub struct RecuperatorGivenOutlet<Fluid, Thermo> {
    thermo: Thermo,
    segments: usize,
    _fluid: PhantomData<Fluid>,
}

/// Specifies which stream's outlet temperature is known.
#[derive(Debug, Clone, Copy)]
pub enum OutletTemp {
    /// The top stream outlet temperature.
    Top(ThermodynamicTemperature),

    /// The bottom stream outlet temperature.
    Bottom(ThermodynamicTemperature),
}

/// Inputs for [`RecuperatorGivenOutlet`].
#[derive(Debug, Clone)]
pub struct RecuperatorGivenOutletInput<Fluid> {
    /// Inlet states for top and bottom streams.
    pub inlets: Inlets<Fluid, Fluid>,

    /// Mass flow rates for top and bottom streams (strictly positive).
    pub mass_flows: MassFlows,

    /// Pressure drops for top and bottom streams (non-negative).
    pub pressure_drops: PressureDrops,

    /// The known outlet temperature and which stream it belongs to.
    pub outlet_temp: OutletTemp,
}

/// Outputs from [`RecuperatorGivenOutlet`].
#[derive(Debug, Clone)]
pub struct RecuperatorGivenOutletOutput<Fluid> {
    /// Top stream outlet state.
    pub top_outlet: State<Fluid>,

    /// Bottom stream outlet state.
    pub bottom_outlet: State<Fluid>,

    /// Heat transfer rate.
    pub q_dot: HeatTransferRate,

    /// Computed overall thermal conductance.
    pub ua: ThermalConductance,

    /// Heat exchanger effectiveness.
    pub effectiveness: Effectiveness,

    /// Minimum hot-to-cold temperature difference and its location.
    pub min_delta_t: MinDeltaT,
}

/// Errors from [`RecuperatorGivenOutlet`] construction and solving.
#[derive(Debug, Error)]
pub enum RecuperatorGivenOutletError {
    /// The segment count is not supported.
    #[error("unsupported segment count {0}; supported values are 1, 5, 10, 20, 50")]
    UnsupportedSegments(usize),

    /// A thermodynamic model operation failed.
    #[error("thermodynamic model failed: {context}")]
    ThermoModelFailed {
        /// Operation context for the thermodynamic model failure.
        context: String,

        /// Underlying thermodynamic model error.
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    /// The specified outlet state violates the second law.
    #[error("second law violation: {message}")]
    SecondLawViolation {
        /// Details about the violation.
        message: String,
    },
}

impl<Fluid, Thermo> RecuperatorGivenOutlet<Fluid, Thermo> {
    /// Creates a discretized counterflow recuperator that solves for UA
    /// given an outlet temperature.
    ///
    /// `thermo` provides thermodynamic property evaluation.
    /// `segments` controls discretization fidelity (see [struct docs](Self)).
    ///
    /// # Errors
    ///
    /// Returns [`RecuperatorGivenOutletError::UnsupportedSegments`] if
    /// `segments` is not in `{1, 5, 10, 20, 50}`.
    pub fn new(thermo: Thermo, segments: usize) -> Result<Self, RecuperatorGivenOutletError> {
        if !matches!(segments, 1 | 5 | 10 | 20 | 50) {
            return Err(RecuperatorGivenOutletError::UnsupportedSegments(segments));
        }

        Ok(Self {
            thermo,
            segments,
            _fluid: PhantomData,
        })
    }

    fn solve<const N: usize>(
        &self,
        input: &RecuperatorGivenOutletInput<Fluid>,
    ) -> Result<RecuperatorGivenOutletOutput<Fluid>, RecuperatorGivenOutletError>
    where
        Fluid: Clone,
        Thermo: DiscretizedHxThermoModel<Fluid>,
    {
        let known = Known {
            inlets: input.inlets.clone(),
            m_dot: input.mass_flows,
            dp: input.pressure_drops,
        };

        let given = match input.outlet_temp {
            OutletTemp::Top(t) => Given::TopOutletTemp(t),
            OutletTemp::Bottom(t) => Given::BottomOutletTemp(t),
        };

        let results = DiscretizedHx::<CounterFlow, N>::solve_same(&known, given, &self.thermo)
            .map_err(RecuperatorGivenOutletError::from)?;

        Ok(Self::to_output(&results))
    }

    fn to_output<const N: usize>(
        results: &Results<Fluid, Fluid, N>,
    ) -> RecuperatorGivenOutletOutput<Fluid>
    where
        Fluid: Clone,
    {
        RecuperatorGivenOutletOutput {
            top_outlet: results.top[N - 1].clone(),
            bottom_outlet: results.bottom[0].clone(),
            q_dot: results.q_dot,
            ua: results.ua,
            effectiveness: results.effectiveness,
            min_delta_t: results.min_delta_t,
        }
    }
}

impl<Fluid, Thermo> Model for RecuperatorGivenOutlet<Fluid, Thermo>
where
    Fluid: Clone,
    Thermo: DiscretizedHxThermoModel<Fluid>,
{
    type Input = RecuperatorGivenOutletInput<Fluid>;
    type Output = RecuperatorGivenOutletOutput<Fluid>;
    type Error = RecuperatorGivenOutletError;

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

impl From<SolveError> for RecuperatorGivenOutletError {
    /// Intentionally flattens structured solver fields into a message.
    /// The recuperator API presents domain-level errors; callers needing
    /// the raw violation details can use the core `DiscretizedHx` API.
    fn from(value: SolveError) -> Self {
        match value {
            SolveError::ThermoModelFailed { context, source } => {
                Self::ThermoModelFailed { context, source }
            }
            SolveError::SecondLawViolation { .. } => Self::SecondLawViolation {
                message: value.to_string(),
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

    fn input_top(
        top: f64,
        bottom: f64,
        outlet_temp: f64,
    ) -> RecuperatorGivenOutletInput<TestFluid> {
        RecuperatorGivenOutletInput {
            inlets: Inlets {
                top: state(top),
                bottom: state(bottom),
            },
            mass_flows: mass_flows(),
            pressure_drops: PressureDrops::default(),
            outlet_temp: OutletTemp::Top(ThermodynamicTemperature::new::<kelvin>(outlet_temp)),
        }
    }

    fn input_bottom(
        top: f64,
        bottom: f64,
        outlet_temp: f64,
    ) -> RecuperatorGivenOutletInput<TestFluid> {
        RecuperatorGivenOutletInput {
            inlets: Inlets {
                top: state(top),
                bottom: state(bottom),
            },
            mass_flows: mass_flows(),
            pressure_drops: PressureDrops::default(),
            outlet_temp: OutletTemp::Bottom(ThermodynamicTemperature::new::<kelvin>(outlet_temp)),
        }
    }

    #[test]
    fn new_accepts_supported_segment_counts() {
        for n in [1, 5, 10, 20, 50] {
            assert!(
                RecuperatorGivenOutlet::<TestFluid, _>::new(thermo(), n).is_ok(),
                "segment count {n} should be accepted",
            );
        }
    }

    #[test]
    fn new_rejects_unsupported_segment_counts() {
        for n in [0, 2, 3, 100] {
            assert!(
                matches!(
                    RecuperatorGivenOutlet::<TestFluid, _>::new(thermo(), n),
                    Err(RecuperatorGivenOutletError::UnsupportedSegments(_))
                ),
                "segment count {n} should be rejected",
            );
        }
    }

    #[test]
    fn top_outlet_computes_ua_and_bottom_outlet() {
        // Top inlet 400 K, bottom inlet 600 K.
        // Specify top outlet at 500 K (heated by 100 K).
        // With equal mass flows and constant cp, bottom cools by 100 K → 500 K.
        let inp = input_top(400.0, 600.0, 500.0);

        let recuperator = RecuperatorGivenOutlet::new(thermo(), 10).unwrap();
        let out = recuperator.call(&inp).unwrap();

        assert_relative_eq!(out.top_outlet.temperature.get::<kelvin>(), 500.0);
        assert_relative_eq!(out.bottom_outlet.temperature.get::<kelvin>(), 500.0);
        assert!(
            out.ua.get::<watt_per_kelvin>() > 0.0,
            "UA should be positive"
        );
    }

    #[test]
    fn bottom_outlet_computes_ua_and_top_outlet() {
        // Top inlet 400 K, bottom inlet 600 K.
        // Specify bottom outlet at 500 K (cooled by 100 K).
        // With equal mass flows and constant cp, top heats by 100 K → 500 K.
        let inp = input_bottom(400.0, 600.0, 500.0);

        let recuperator = RecuperatorGivenOutlet::new(thermo(), 10).unwrap();
        let out = recuperator.call(&inp).unwrap();

        assert_relative_eq!(out.top_outlet.temperature.get::<kelvin>(), 500.0);
        assert_relative_eq!(out.bottom_outlet.temperature.get::<kelvin>(), 500.0);
        assert!(
            out.ua.get::<watt_per_kelvin>() > 0.0,
            "UA should be positive"
        );
    }

    #[test]
    fn outlet_at_inlet_temp_gives_zero_ua() {
        // Top outlet = top inlet → no heat transfer → UA = 0.
        let inp = input_top(400.0, 600.0, 400.0);

        let recuperator = RecuperatorGivenOutlet::new(thermo(), 10).unwrap();
        let out = recuperator.call(&inp).unwrap();

        assert_relative_eq!(out.ua.get::<watt_per_kelvin>(), 0.0);
        assert_relative_eq!(out.top_outlet.temperature.get::<kelvin>(), 400.0);
        assert_relative_eq!(out.bottom_outlet.temperature.get::<kelvin>(), 600.0);
    }

    #[test]
    fn given_ua_and_given_outlet_agree() {
        // Solve with GivenUa, then use the outlet temp with GivenOutlet.
        // Both should produce the same UA.
        use crate::models::thermal::hx::discretized::recuperator::given_ua::{
            RecuperatorGivenUa, RecuperatorGivenUaConfig, RecuperatorGivenUaInput,
        };

        let target_ua = ThermalConductance::new::<watt_per_kelvin>(500.0);

        let ua_model =
            RecuperatorGivenUa::new(thermo(), 10, RecuperatorGivenUaConfig::default()).unwrap();
        let ua_result = ua_model
            .call(&RecuperatorGivenUaInput {
                inlets: Inlets {
                    top: state(400.0),
                    bottom: state(600.0),
                },
                mass_flows: mass_flows(),
                pressure_drops: PressureDrops::default(),
                ua: target_ua,
            })
            .unwrap();

        // Now use the top outlet temperature from the UA solve.
        let outlet_model = RecuperatorGivenOutlet::new(thermo(), 10).unwrap();
        let outlet_result = outlet_model
            .call(&RecuperatorGivenOutletInput {
                inlets: Inlets {
                    top: state(400.0),
                    bottom: state(600.0),
                },
                mass_flows: mass_flows(),
                pressure_drops: PressureDrops::default(),
                outlet_temp: OutletTemp::Top(ua_result.top_outlet.temperature),
            })
            .unwrap();

        assert_relative_eq!(
            outlet_result.ua.get::<watt_per_kelvin>(),
            ua_result.ua.get::<watt_per_kelvin>(),
            epsilon = 1e-6,
        );
    }
}
