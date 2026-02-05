//! Public Twine models.
//!
//! Models are the primary public interface of this crate.
//!
//! # Organization
//!
//! Models are organized into domain-specific submodules (e.g., `turbomachinery`,
//! `controllers`, `thermal`) based on an opinionated taxonomy. This organization
//! may evolve as more models are added.
//!
//! # Model structure
//!
//! Each model lives in its own module and contains an internal `core` submodule
//! where the actual computation and domain logic lives. The `core` module is an
//! implementation detail and is **not** re-exported as part of the public API.
//!
//! The [`twine_core::Model`] implementation should be a thin adapter that
//! delegates to the model-specific core API. A single `core` may be exposed
//! through multiple adapters (e.g., different wrapper types implementing `Model`).
