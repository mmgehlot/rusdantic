//! Core validation traits.
//!
//! The [`Validate`] trait is the central abstraction of Rusdantic. It is
//! automatically implemented by the `#[derive(Rusdantic)]` macro and can
//! also be implemented manually for custom types.

use crate::error::ValidationErrors;

/// The core validation trait for Rusdantic.
///
/// This trait is automatically implemented by `#[derive(Rusdantic)]`.
/// It validates all fields of a struct according to their declared rules
/// and returns all validation errors at once (collect-all, not fail-fast).
///
/// # Example
///
/// ```ignore
/// use rusdantic::{Rusdantic, Validate};
///
/// #[derive(Rusdantic)]
/// struct User {
///     #[rusdantic(email)]
///     email: String,
/// }
///
/// let user = User { email: "not-an-email".to_string() };
/// assert!(user.validate().is_err());
/// ```
pub trait Validate {
    /// Validate this value, collecting all validation errors.
    ///
    /// Returns `Ok(())` if all validation rules pass, or `Err(ValidationErrors)`
    /// containing all validation failures with their field paths.
    fn validate(&self) -> Result<(), ValidationErrors>;

    /// Validate with external context (database connection, config, etc.).
    ///
    /// This method enables validators that need access to external resources.
    /// The default implementation ignores the context and delegates to `validate()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct DbContext { /* ... */ }
    ///
    /// let user = User { email: "user@example.com".to_string() };
    /// let ctx = DbContext { /* ... */ };
    /// user.validate_with_context(&ctx)?;
    /// ```
    fn validate_with_context<C>(&self, _ctx: &C) -> Result<(), ValidationErrors> {
        self.validate()
    }
}

// Implement Validate for common wrapper types so they can be used
// transparently in validated structs.

impl<T: Validate> Validate for Box<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        (**self).validate()
    }
}

impl<T: Validate> Validate for std::sync::Arc<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        (**self).validate()
    }
}

impl<T: Validate> Validate for std::rc::Rc<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        (**self).validate()
    }
}
