//! # Twine Models
//!
//! Opinionated, domain-specific models and model-building tools for
//! [Twine](https://github.com/isentropic-dev/twine).
//!
//! ## Crate layout
//!
//! - [`models`]: Public, domain-specific [`twine_core::Model`] implementations.
//! - [`support`]: Supporting utilities used by models.
//!
//! ## Stability and code lifecycle
//!
//! Modules in [`support`] are published because they're useful, but their APIs
//! are not stable. Breaking changes may occur as needed.
//!
//! Code in this crate follows a natural progression as needs emerge:
//!
//! 1. **Model-specific**: Starts in a model's internal `core` module
//! 2. **Domain-specific**: If useful across models in a domain (e.g., `turbomachinery`),
//!    it may be promoted to a domain-level support module
//! 3. **Crate-level**: If useful across multiple domains or potentially useful
//!    outside this crate, it may move to [`support`]
//! 4. **Standalone**: If broadly useful and stable, it may become its own crate
//!    (and be removed here in a future release)

pub mod models;
pub mod support;
