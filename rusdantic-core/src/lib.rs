//! # rusdantic-core
//!
//! Core validation traits, error types, and built-in validation rules for the
//! Rusdantic framework. This crate is the runtime component — it contains
//! everything that the derive macro's generated code calls at runtime.
//!
//! Users should not depend on this crate directly. Instead, use the `rusdantic`
//! facade crate which re-exports everything needed.

#![warn(missing_docs)]

pub mod coerce;
pub mod dump;
pub mod error;
pub mod rules;
pub mod traits;

// Re-export core types at the crate root for ergonomic use in generated code.
// The derive macro generates code like `::rusdantic_core::Validate`,
// `::rusdantic_core::ValidationErrors`, etc.
pub use error::{PathSegment, ValidationError, ValidationErrors};
pub use traits::Validate;

/// Re-export regex for use in generated code (OnceLock<Regex> statics).
/// This avoids requiring users to add `regex` as a direct dependency.
pub mod re_export {
    pub use regex::Regex;
}
