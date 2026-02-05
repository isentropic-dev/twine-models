//! Public Twine models.
//!
//! Models are the primary public interface of this crate.
//!
//! # Organization
//!
//! Models are organized into domain-specific submodules (e.g., `thermal`,
//! `turbomachinery`, `controllers`) based on an opinionated taxonomy.
//! This organization may evolve as more models are added.
//!
//! # Model structure
//!
//! Related models are grouped in their own modules. Each model module contains an
//! internal `core` submodule where the actual computation and domain logic lives.
//! The `core` module is an implementation detail and is **not** part of the
//! model's public API.
//!
//! The [`twine_core::Model`] implementation should be a thin adapter that
//! delegates to the model-specific core API. A single `core` may be exposed
//! through multiple adapters (e.g., different wrapper types implementing
//! [`twine_core::Model`]).
