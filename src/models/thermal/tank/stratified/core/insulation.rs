/// Options for specifying tank insulation.
///
/// The insulation setting determines what conductance value is applied between
/// the tank fluid and the surrounding environment on all exposed surfaces.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Insulation {
    /// The tank is perfectly insulated — no heat transfer to the environment.
    Adiabatic,
}
