//! Numeric range validation rule.
//!
//! Validates that a numeric value falls within the specified bounds.
//! Works with any type that implements `PartialOrd` and `Display`.

use crate::error::{PathSegment, ValidationError, ValidationErrors};
use std::fmt::Display;

/// Validate that the value is within the specified numeric range.
///
/// - `min`: Minimum value (inclusive). `None` means no lower bound.
/// - `max`: Maximum value (inclusive). `None` means no upper bound.
///
/// Works with all Rust numeric types: i8, i16, i32, i64, i128, u8, u16, u32,
/// u64, u128, f32, f64, isize, usize.
pub fn validate_range<T: PartialOrd + Display + Into<serde_json::Value> + Copy>(
    value: &T,
    min: Option<T>,
    max: Option<T>,
    path: &[PathSegment],
    errors: &mut ValidationErrors,
) {
    if let Some(min_val) = min {
        if *value < min_val {
            errors.add(
                ValidationError::new(
                    "range_min",
                    format!("must be at least {}", min_val),
                )
                .with_path(path.to_vec())
                .with_param("min", (*value).into())
                .with_param("actual", (*value).into()),
            );
        }
    }

    if let Some(max_val) = max {
        if *value > max_val {
            errors.add(
                ValidationError::new(
                    "range_max",
                    format!("must be at most {}", max_val),
                )
                .with_path(path.to_vec())
                .with_param("max", (*value).into())
                .with_param("actual", (*value).into()),
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
    fn test_u8_range_valid() {
        let mut errors = ValidationErrors::new();
        validate_range(&25u8, Some(18u8), Some(120u8), &path("age"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_u8_range_below_min() {
        let mut errors = ValidationErrors::new();
        validate_range(&16u8, Some(18u8), None, &path("age"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "range_min");
    }

    #[test]
    fn test_u8_range_above_max() {
        let mut errors = ValidationErrors::new();
        validate_range(&200u8, None, Some(150u8), &path("age"), &mut errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "range_max");
    }

    #[test]
    fn test_i32_range() {
        let mut errors = ValidationErrors::new();
        validate_range(&-5i32, Some(-10i32), Some(10i32), &path("temp"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_i64_range_negative() {
        let mut errors = ValidationErrors::new();
        validate_range(&-20i64, Some(-10i64), None, &path("offset"), &mut errors);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_f64_range() {
        let mut errors = ValidationErrors::new();
        validate_range(&3.14f64, Some(0.0f64), Some(10.0f64), &path("ratio"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_f64_range_below() {
        let mut errors = ValidationErrors::new();
        validate_range(&-0.1f64, Some(0.0f64), None, &path("ratio"), &mut errors);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_boundary_values() {
        let mut errors = ValidationErrors::new();
        // Exactly at min should be valid
        validate_range(&18u8, Some(18u8), Some(120u8), &path("age"), &mut errors);
        assert!(errors.is_empty());

        // Exactly at max should be valid
        validate_range(&120u8, Some(18u8), Some(120u8), &path("age"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_no_bounds() {
        let mut errors = ValidationErrors::new();
        validate_range::<i32>(&999, None, None, &path("f"), &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_usize_range() {
        let mut errors = ValidationErrors::new();
        validate_range(&5usize, Some(1usize), Some(100usize), &path("count"), &mut errors);
        assert!(errors.is_empty());
    }
}
