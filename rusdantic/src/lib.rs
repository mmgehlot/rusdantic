//! # Rusdantic
//!
//! A high-ergonomics data validation and serialization framework for Rust,
//! inspired by Python's [Pydantic](https://docs.pydantic.dev).
//!
//! Rusdantic bridges the gap between [serde](https://serde.rs) (serialization)
//! and validation crates like [validator](https://crates.io/crates/validator) /
//! [garde](https://crates.io/crates/garde) into a single, unified derive macro.
//!
//! ## Quick Start
//!
//! ```rust
//! use rusdantic::prelude::*;
//!
//! #[derive(Rusdantic, Debug)]
//! struct User {
//!     #[rusdantic(length(min = 3, max = 20))]
//!     username: String,
//!
//!     #[rusdantic(email)]
//!     email: String,
//!
//!     #[rusdantic(range(min = 18))]
//!     age: u8,
//! }
//!
//! // Deserialize + validate in one step
//! let json = r#"{"username": "rust_ace", "email": "user@example.com", "age": 25}"#;
//! let user: User = rusdantic::from_json(json).unwrap();
//!
//! // Or validate a manually constructed struct
//! let user = User {
//!     username: "ab".to_string(),
//!     email: "not-email".to_string(),
//!     age: 16,
//! };
//! assert!(user.validate().is_err());
//! ```
//!
//! ## Features
//!
//! - **Unified Derive Macro**: `#[derive(Rusdantic)]` generates `Serialize`,
//!   `Deserialize`, and `Validate` in one shot.
//! - **Validate-on-Deserialize**: Validation runs during deserialization,
//!   so invalid structs never exist in memory when using `from_json()`.
//! - **Path-Aware Errors**: Get precise error paths like
//!   `user.addresses[0].zip_code` for nested structures.
//! - **7 Built-in Validators**: `length`, `range`, `email`, `url`, `pattern`,
//!   `contains`, `required` — plus custom validators.
//! - **Serde Compatibility**: Works with `#[serde(rename)]`, `#[serde(default)]`,
//!   and other serde attributes.
//! - **JSON Schema Generation**: Generate Draft 2020-12 / OpenAPI 3.1 schemas.
//! - **PII Redaction**: `#[rusdantic(redact)]` hides sensitive data in Debug output.
//! - **Zero-Cost Abstractions**: Validation logic is monomorphized at compile time.

#![warn(missing_docs)]

// Re-export the derive macro from rusdantic-derive
pub use rusdantic_derive::Rusdantic;

// Re-export core types so users only need `use rusdantic::*` or `use rusdantic::prelude::*`
pub use rusdantic_core::{PathSegment, Validate, ValidationError, ValidationErrors};

/// Error type for Rusdantic operations that may fail due to either
/// deserialization errors (malformed JSON) or validation errors
/// (valid JSON but invalid data).
#[derive(Debug)]
pub enum RusdanticError {
    /// A serde deserialization error (malformed input, wrong types, missing fields).
    Deserialization(serde_json::Error),
    /// Validation errors (correct structure but invalid data values).
    Validation(ValidationErrors),
}

impl std::fmt::Display for RusdanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RusdanticError::Deserialization(e) => write!(f, "deserialization error: {}", e),
            RusdanticError::Validation(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for RusdanticError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RusdanticError::Deserialization(e) => Some(e),
            RusdanticError::Validation(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for RusdanticError {
    fn from(err: serde_json::Error) -> Self {
        RusdanticError::Deserialization(err)
    }
}

impl From<ValidationErrors> for RusdanticError {
    fn from(err: ValidationErrors) -> Self {
        RusdanticError::Validation(err)
    }
}

/// Deserialize a JSON string into a validated struct.
///
/// This is the primary entry point for Rusdantic. It combines serde
/// deserialization with validation in a single step. If the `#[derive(Rusdantic)]`
/// macro was used, validation is embedded in the `Deserialize` impl, so
/// this function returns an error if either deserialization or validation fails.
///
/// # Errors
///
/// Returns `RusdanticError::Deserialization` if the JSON is malformed or
/// has wrong types. Returns `RusdanticError::Validation` (embedded in the
/// serde error) if the data is structurally correct but fails validation.
///
/// # Example
///
/// ```rust
/// use rusdantic::prelude::*;
///
/// #[derive(Rusdantic, Debug)]
/// struct Config {
///     #[rusdantic(length(min = 1))]
///     name: String,
///     #[rusdantic(range(min = 1, max = 65535))]
///     port: u16,
/// }
///
/// let config: Config = rusdantic::from_json(
///     r#"{"name": "my-app", "port": 8080}"#
/// ).unwrap();
/// ```
pub fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, RusdanticError> {
    serde_json::from_str(json).map_err(RusdanticError::Deserialization)
}

/// Deserialize a `serde_json::Value` into a validated struct.
///
/// Similar to [`from_json`] but accepts a pre-parsed JSON value instead
/// of a raw string.
///
/// # Example
///
/// ```rust
/// use rusdantic::prelude::*;
/// use serde_json::json;
///
/// #[derive(Rusdantic, Debug)]
/// struct Item {
///     #[rusdantic(length(min = 1))]
///     name: String,
/// }
///
/// let value = json!({"name": "widget"});
/// let item: Item = rusdantic::from_value(value).unwrap();
/// ```
pub fn from_value<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, RusdanticError> {
    serde_json::from_value(value).map_err(RusdanticError::Deserialization)
}

/// Convenience prelude module for common imports.
///
/// ```rust
/// use rusdantic::prelude::*;
/// ```
///
/// This imports:
/// - `Rusdantic` derive macro
/// - `Validate` trait
/// - `ValidationError` and `ValidationErrors` types
/// - `PathSegment` enum
pub mod prelude {
    pub use crate::{PathSegment, Rusdantic, Validate, ValidationError, ValidationErrors};
}
