//! Length validation rule.
//!
//! Validates that a value's length (character count for strings, element count
//! for collections) falls within the specified bounds.

use crate::error::{PathSegment, ValidationError, ValidationErrors};
use crate::rules::HasLength;

/// Validate that the value's length is within the specified bounds.
///
/// - `min`: Minimum length (inclusive). `None` means no lower bound.
/// - `max`: Maximum length (inclusive). `None` means no upper bound.
///
/// Adds errors to `errors` if the length is out of bounds.
/// Uses character count (not byte length) for strings to handle Unicode correctly.
pub fn validate_length<T: HasLength>(
    value: &T,
    min: Option<usize>,
    max: Option<usize>,
    path: &[PathSegment],
    errors: &mut ValidationErrors,
) {
    let actual = value.rusdantic_length();
    // Determine the unit for error messages based on the type.
    // HasLength is implemented for String/&str (characters) and collections (items).
    let unit = value.rusdantic_length_unit();

    if let Some(min_val) = min {
        if actual < min_val {
            errors.add(
                ValidationError::new(
                    "length_min",
                    format!("must be at least {} {}", min_val, unit),
                )
                .with_path(path.to_vec())
                .with_param("min", serde_json::json!(min_val))
                .with_param("actual", serde_json::json!(actual)),
            );
        }
    }

    if let Some(max_val) = max {
        if actual > max_val {
            errors.add(
                ValidationError::new(
                    "length_max",
                    format!("must be at most {} {}", max_val, unit),
                )
                .with_path(path.to_vec())
                .with_param("max", serde_json::json!(max_val))
                .with_param("actual", serde_json::json!(actual)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> Vec<PathSegment> {
        vec![PathSegment::Field(name.to_string())]
    }

    #[test]
    fn test_string_length_valid() {
        let mut errors = ValidationErrors::new();
        validate_length(&"hello".to_string(), Some(1), Some(10), &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_length_too_short() {
        let mut errors = ValidationErrors::new();
        validate_length(&"ab".to_string(), Some(3), None, &path("f"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "length_min");
        assert_eq!(errors.errors()[0].params["min"], 3);
        assert_eq!(errors.errors()[0].params["actual"], 2);
    }

    #[test]
    fn test_string_length_too_long() {
        let mut errors = ValidationErrors::new();
        validate_length(
            &"hello world".to_string(),
            None,
            Some(5),
            &path("f"),
            &mut errors,
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "length_max");
    }

    #[test]
    fn test_string_length_both_bounds_violated() {
        let mut errors = ValidationErrors::new();
        validate_length(&"".to_string(), Some(1), Some(10), &path("f"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "length_min");
    }

    #[test]
    fn test_string_length_unicode() {
        // "café" has 4 characters but 5 bytes in UTF-8
        let mut errors = ValidationErrors::new();
        validate_length(&"café".to_string(), Some(1), Some(4), &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_string_length_emoji() {
        // Emoji characters should count as 1 character each
        let mut errors = ValidationErrors::new();
        validate_length(&"🦀🐍".to_string(), Some(1), Some(2), &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_vec_length_valid() {
        let mut errors = ValidationErrors::new();
        validate_length(&vec![1, 2, 3], Some(1), Some(5), &path("items"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_vec_length_too_short() {
        let mut errors = ValidationErrors::new();
        let v: Vec<i32> = vec![];
        validate_length(&v, Some(1), None, &path("items"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "length_min");
    }

    #[test]
    fn test_hashmap_length() {
        use std::collections::HashMap;
        let mut errors = ValidationErrors::new();
        let mut map = HashMap::new();
        map.insert("a", 1);
        validate_length(&map, Some(1), Some(5), &path("data"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_no_bounds() {
        let mut errors = ValidationErrors::new();
        validate_length(&"anything".to_string(), None, None, &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_exact_length() {
        let mut errors = ValidationErrors::new();
        validate_length(&"abc".to_string(), Some(3), Some(3), &path("f"), &mut errors);
        assert!(errors.is_empty());
    }
}
