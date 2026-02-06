//! # Twine Models
//!
//! Opinionated, domain-specific models and model-building tools for
//! [Twine](https://github.com/isentropic-dev/twine).
//!
//! ## Crate layout
//!
//! - [`models`]: Domain-specific [`twine_core::Model`] implementations.
//! - [`support`]: Supporting utilities used by models.
//!
//! ## Utility code lifecycle
//!
//! Modules in [`support`] are part of the public API because they're useful,
//! but their APIs are not stable. Breaking changes may occur as needed.
//!
//! Utility code in this crate follows a natural progression as needs emerge:
//!
//! 1. **Model-specific**: Starts in a model's internal `core` module
//! 2. **Domain-specific**: If useful across models in a domain (e.g., `turbomachinery`),
//!    it moves to a domain-level support module
//! 3. **Crate-level**: If useful across multiple domains or potentially useful
//!    outside this crate, it moves to [`support`]
//! 4. **Standalone**: If broadly useful and stable, it may become its own crate
//!    (and be removed from here in a future release)
//!
//! Note: Only utilities at the crate-level (in [`support`]) are part of the public API.
//! Model-specific and domain-specific utility code remains private.

pub mod models;
pub mod support;
