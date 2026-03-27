//! Required field validation rule.
//!
//! Validates that an `Option<T>` field is `Some`, not `None`.
//! This is used when a field is typed as `Option<T>` for serde compatibility
//! (allowing the key to be omitted from JSON) but the business logic requires
//! the value to be present.

use crate::error::{PathSegment, ValidationError, ValidationErrors};

/// Validate that an `Option<T>` value is `Some` (not `None`).
///
/// This validator is designed for use with `Option<T>` fields that have the
/// `#[rusdantic(required)]` attribute. It checks the raw `Option` before
/// any unwrapping, so it must be called with the original `Option` value.
///
/// # Type flexibility
///
/// The generic parameter allows this to work with `Option<T>` for any `T`.
/// The caller must pass the `Option` itself, not the inner value.
pub fn validate_required<T>(
    value: &Option<T>,
    path: &[PathSegment],
    errors: &mut ValidationErrors,
) {
    if value.is_none() {
        errors.add(
            ValidationError::new("required", "this field is required")
                .with_path(path.to_vec()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> Vec<PathSegment> {
        vec![PathSegment::Field(name.to_string())]
    }

    #[test]
    fn test_required_some() {
        let mut errors = ValidationErrors::new();
        validate_required(&Some("value".to_string()), &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_required_none() {
        let mut errors = ValidationErrors::new();
        validate_required::<String>(&None, &path("f"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "required");
        assert_eq!(errors.errors()[0].message, "this field is required");
    }

    #[test]
    fn test_required_some_integer() {
        let mut errors = ValidationErrors::new();
        validate_required(&Some(42), &path("count"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_required_none_integer() {
        let mut errors = ValidationErrors::new();
        validate_required::<i32>(&None, &path("count"), &mut errors);
        assert_eq!(errors.len(), 1);
    }
}
