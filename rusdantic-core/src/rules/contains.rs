//! String contains validation rule.
//!
//! Validates that a string value contains a specified substring.

use crate::error::{PathSegment, ValidationError, ValidationErrors};
use crate::rules::AsStr;

/// Validate that the value contains the specified substring.
///
/// This is a case-sensitive check. For case-insensitive matching,
/// use a pattern validator with an appropriate regex instead.
pub fn validate_contains<T: AsStr>(
    value: &T,
    needle: &str,
    path: &[PathSegment],
    errors: &mut ValidationErrors,
) {
    let s = value.as_str_ref();

    if !s.contains(needle) {
        let display_needle = if needle.len() > 50 {
            format!("{}...", &needle[..50])
        } else {
            needle.to_string()
        };
        errors.add(
            ValidationError::new("contains", format!("must contain '{}'", display_needle))
                .with_path(path.to_vec())
                .with_param("expected", serde_json::json!(needle)),
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
    fn test_contains_present() {
        let mut errors = ValidationErrors::new();
        validate_contains(&"hello@world.com".to_string(), "@", &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_contains_absent() {
        let mut errors = ValidationErrors::new();
        validate_contains(&"hello".to_string(), "@", &path("f"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "contains");
        assert_eq!(errors.errors()[0].params["expected"], "@");
    }

    #[test]
    fn test_contains_case_sensitive() {
        let mut errors = ValidationErrors::new();
        validate_contains(&"Hello".to_string(), "hello", &path("f"), &mut errors);
        assert_eq!(errors.len(), 1); // Case-sensitive: "Hello" does not contain "hello"
    }

    #[test]
    fn test_contains_empty_needle() {
        let mut errors = ValidationErrors::new();
        // Empty string is always contained in any string
        validate_contains(&"anything".to_string(), "", &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_contains_empty_value() {
        let mut errors = ValidationErrors::new();
        validate_contains(&"".to_string(), "x", &path("f"), &mut errors);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_contains_unicode() {
        let mut errors = ValidationErrors::new();
        validate_contains(&"café latte".to_string(), "café", &path("f"), &mut errors);
        assert!(errors.is_empty());
    }
}
