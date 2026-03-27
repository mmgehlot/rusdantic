//! # rusdantic-types
//!
//! Extended type library for the Rusdantic framework providing constrained
//! types that validate their values at construction time and during
//! deserialization. These types are zero-cost newtypes that enforce
//! constraints through Rust's type system.
//!
//! ## Categories
//!
//! - **Numeric**: `PositiveInt`, `NegativeInt`, `NonNegativeInt`, `FiniteFloat`
//! - **String**: `NonEmptyString`, `EmailStr`
//! - **Secret**: `SecretStr`, `SecretBytes` (redacted in Debug/Display)
//! - **Network**: `HttpUrl`

#![warn(missing_docs)]

pub mod network;
pub mod numeric;
pub mod secret;
pub mod string;

// Re-export all types at the crate root
pub use network::*;
pub use numeric::*;
pub use secret::*;
pub use string::*;
