//! Regex pattern validation rule.
//!
//! Validates that a string value matches a given regex pattern.
//! The regex is compiled once at first use via OnceLock in the generated code
//! and passed as a reference to this validator.

use crate::error::{PathSegment, ValidationError, ValidationErrors};
use crate::rules::AsStr;

/// Validate that the value matches the given regex pattern.
///
/// The `regex` parameter is a pre-compiled `Regex` object (managed by
/// an `OnceLock` static in the generated code). The `pattern_str` parameter
/// is the raw regex string, included in error messages for debugging.
///
/// The pattern is matched against the entire string — it is automatically
/// anchored. If the original regex does not anchor, this function still
/// checks that the full string matches (via `is_match`). If partial matching
/// is desired, the regex should use `.*` appropriately.
/// Ensure a regex pattern matches the FULL string (not partial).
///
/// If the pattern doesn't start with `^` and end with `$`, it's wrapped
/// in `^(?:...)$` to ensure full-string matching. This prevents patterns
/// like `[0-9]{5}` from matching within "abc12345xyz".
///
/// Wraps the pattern in a non-capturing group `(?:...)` to avoid changing
/// the numbering of any capture groups in the user's original pattern.
pub fn anchor_pattern(pattern: &str) -> String {
    let starts = pattern.starts_with('^');
    let ends = pattern.ends_with('$');
    match (starts, ends) {
        (true, true) => pattern.to_string(),
        (true, false) => format!("{}$", pattern),
        (false, true) => format!("^{}", pattern),
        (false, false) => format!("^(?:{})$", pattern),
    }
}

/// Validate that the value matches the given regex pattern.
///
/// The `regex` parameter is a pre-compiled `Regex` object (managed by
/// an `OnceLock` static in the generated code). The `pattern_str` parameter
/// is the raw regex string, included in error messages for debugging.
pub fn validate_pattern<T: AsStr>(
    value: &T,
    regex: &regex::Regex,
    pattern_str: &str,
    path: &[PathSegment],
    errors: &mut ValidationErrors,
) {
    let s = value.as_str_ref();

    if !regex.is_match(s) {
        let display_pattern = if pattern_str.len() > 50 {
            format!("{}...", &pattern_str[..50])
        } else {
            pattern_str.to_string()
        };
        errors.add(
            ValidationError::new(
                "pattern",
                format!("must match pattern '{}'", display_pattern),
            )
            .with_path(path.to_vec())
            .with_param("pattern", serde_json::json!(pattern_str)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn path(name: &str) -> Vec<PathSegment> {
        vec![PathSegment::Field(name.to_string())]
    }

    #[test]
    fn test_pattern_matches() {
        let re = Regex::new(r"^[a-z]+$").unwrap();
        let mut errors = ValidationErrors::new();
        validate_pattern(
            &"hello".to_string(),
            &re,
            "^[a-z]+$",
            &path("f"),
            &mut errors,
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_pattern_no_match() {
        let re = Regex::new(r"^[a-z]+$").unwrap();
        let mut errors = ValidationErrors::new();
        validate_pattern(
            &"Hello123".to_string(),
            &re,
            "^[a-z]+$",
            &path("f"),
            &mut errors,
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].code, "pattern");
        assert_eq!(errors.errors()[0].params["pattern"], "^[a-z]+$");
    }

    #[test]
    fn test_pattern_with_digits() {
        let re = Regex::new(r"^\d{3}-\d{4}$").unwrap();
        let mut errors = ValidationErrors::new();
        validate_pattern(
            &"123-4567".to_string(),
            &re,
            r"^\d{3}-\d{4}$",
            &path("phone"),
            &mut errors,
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_pattern_empty_string() {
        let re = Regex::new(r"^.+$").unwrap();
        let mut errors = ValidationErrors::new();
        validate_pattern(&"".to_string(), &re, "^.+$", &path("f"), &mut errors);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_pattern_unicode() {
        let re = Regex::new(r"^\p{L}+$").unwrap();
        let mut errors = ValidationErrors::new();
        validate_pattern(
            &"héllo".to_string(),
            &re,
            r"^\p{L}+$",
            &path("f"),
            &mut errors,
        );
        assert!(errors.is_empty());
    }
}
