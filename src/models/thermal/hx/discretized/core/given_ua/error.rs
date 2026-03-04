use thiserror::Error;
use twine_solvers::equation::bisection;
use uom::si::f64::ThermalConductance;

use crate::models::thermal::hx::discretized::core::SolveError;

/// Errors that can occur while solving given a target conductance (UA).
#[derive(Debug, Error)]
pub enum GivenUaError {
    /// The target UA is negative.
    #[error("target UA must be non-negative, got {0:?}")]
    NegativeUa(ThermalConductance),

    /// The inlet temperatures are equal.
    ///
    /// The solver brackets the outlet temperature between the two inlet
    /// temperatures. When they are equal, no bracket can be formed.
    #[error("equal inlet temperatures: solver cannot form a search bracket")]
    EqualInletTemperatures,

    /// A discretized heat exchanger solve failed.
    #[error("discretized solve failed")]
    Solve(#[from] SolveError),

    /// The bisection solver encountered an error.
    #[error("bisection solver error")]
    Bisection(#[from] bisection::Error),

    /// The solver reached the iteration limit without converging.
    #[error("solver hit iteration limit: residual={residual:?}")]
    MaxIters {
        /// Best UA residual achieved.
        ///
        /// This is the smallest absolute difference between achieved and target
        /// conductance encountered during iteration.
        residual: ThermalConductance,

        /// Iteration count performed by the solver.
        iters: usize,
    },
}
