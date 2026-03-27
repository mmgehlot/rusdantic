//! Error types for validation results.
//!
//! Provides [`ValidationError`] for individual field errors and
//! [`ValidationErrors`] for collecting multiple errors across a struct.
//! All types are JSON-serializable for API error responses and implement
//! `Display` for human-readable output.

use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

/// A single validation error with location context.
///
/// Each error includes the path to the invalid field (supporting nested structs
/// and collection indices), a machine-readable error code, a human-readable
/// message, and optional constraint parameters for structured error reporting.
///
/// # Example
///
/// ```
/// use rusdantic_core::{ValidationError, PathSegment};
///
/// let error = ValidationError {
///     path: vec![
///         PathSegment::Field("user".to_string()),
///         PathSegment::Field("email".to_string()),
///     ],
///     code: "email".to_string(),
///     message: "invalid email format".to_string(),
///     params: Default::default(),
/// };
///
/// assert_eq!(error.path_string(), "user.email");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidationError {
    /// Path to the invalid field.
    ///
    /// For nested structs, this contains multiple segments:
    /// `["user", "addresses", 0, "zip_code"]`
    pub path: Vec<PathSegment>,

    /// Machine-readable error code.
    ///
    /// Standard codes include: `"length_min"`, `"length_max"`, `"range_min"`,
    /// `"range_max"`, `"email"`, `"url"`, `"pattern"`, `"contains"`, `"required"`,
    /// `"custom"`.
    pub code: String,

    /// Human-readable error message suitable for display to end users.
    pub message: String,

    /// Constraint parameters providing context about the validation failure.
    ///
    /// For example, a length violation might include:
    /// `{"min": 3, "max": 20, "actual": 1}`
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub params: HashMap<String, serde_json::Value>,
}

impl ValidationError {
    /// Create a new validation error with the given code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: Vec::new(),
            code: code.into(),
            message: message.into(),
            params: HashMap::new(),
        }
    }

    /// Add a parameter to this error for structured error reporting.
    pub fn with_param(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Set the path for this error.
    pub fn with_path(mut self, path: Vec<PathSegment>) -> Self {
        self.path = path;
        self
    }

    /// Format the path as a dot-notation string.
    ///
    /// Field segments are joined with `.`, index segments are formatted as `[N]`.
    /// Example: `"user.addresses[0].zip_code"`
    pub fn path_string(&self) -> String {
        let mut result = String::new();
        for (i, segment) in self.path.iter().enumerate() {
            match segment {
                PathSegment::Field(name) => {
                    if i > 0 {
                        result.push('.');
                    }
                    result.push_str(name);
                }
                PathSegment::Index(idx) => {
                    result.push('[');
                    result.push_str(&idx.to_string());
                    result.push(']');
                }
            }
        }
        result
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path_string();
        if path.is_empty() {
            write!(f, "{} ({})", self.message, self.code)
        } else {
            write!(f, "{}: {} ({})", path, self.message, self.code)
        }
    }
}

impl std::error::Error for ValidationError {}

/// Path segment in a validation error location.
///
/// Supports both field names (for struct fields and map keys) and
/// numeric indices (for arrays/vectors).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum PathSegment {
    /// A named field in a struct or map key.
    Field(String),
    /// A numeric index in an array or vector.
    Index(usize),
}

/// A collection of validation errors from validating a struct.
///
/// This type aggregates all validation errors found during validation,
/// allowing users to see all problems at once rather than fixing them
/// one at a time. It implements both JSON serialization (for API responses)
/// and human-readable `Display` (for CLI/logging output).
///
/// # Example
///
/// ```
/// use rusdantic_core::{ValidationErrors, ValidationError, PathSegment};
///
/// let mut errors = ValidationErrors::new();
/// assert!(errors.is_empty());
///
/// errors.add(ValidationError {
///     path: vec![PathSegment::Field("email".to_string())],
///     code: "email".to_string(),
///     message: "invalid email format".to_string(),
///     params: Default::default(),
/// });
///
/// assert_eq!(errors.len(), 1);
/// assert!(!errors.is_empty());
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ValidationErrors {
    /// The collected validation errors.
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Create an empty error collection.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add a single validation error to the collection.
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Merge another `ValidationErrors` into this one.
    /// All errors from `other` are moved into this collection.
    pub fn merge(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    /// Check if no errors have been collected.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Return the number of collected errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Get a reference to the collected errors.
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Consume self and return the inner vector of errors.
    pub fn into_errors(self) -> Vec<ValidationError> {
        self.errors
    }

    /// Get errors for a specific field path (first segment only).
    pub fn field_errors(&self, field_name: &str) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| {
                e.path
                    .first()
                    .map(|s| matches!(s, PathSegment::Field(name) if name == field_name))
                    .unwrap_or(false)
            })
            .collect()
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValidationErrors {
    /// Display all errors in a human-readable format, one per line.
    ///
    /// Example output:
    /// ```text
    /// Validation failed with 2 error(s):
    ///   - username: must be between 3 and 20 characters (length)
    ///   - email: invalid email format (email)
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation failed with {} error(s):", self.errors.len())?;
        for error in &self.errors {
            writeln!(f, "  - {}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

// Implement IntoIterator for ergonomic error iteration
impl IntoIterator for ValidationErrors {
    type Item = ValidationError;
    type IntoIter = std::vec::IntoIter<ValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a> IntoIterator for &'a ValidationErrors {
    type Item = &'a ValidationError;
    type IntoIter = std::slice::Iter<'a, ValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display_with_path() {
        let error = ValidationError {
            path: vec![
                PathSegment::Field("user".to_string()),
                PathSegment::Field("email".to_string()),
            ],
            code: "email".to_string(),
            message: "invalid email format".to_string(),
            params: HashMap::new(),
        };
        assert_eq!(
            error.to_string(),
            "user.email: invalid email format (email)"
        );
    }

    #[test]
    fn test_validation_error_display_with_index() {
        let error = ValidationError {
            path: vec![
                PathSegment::Field("users".to_string()),
                PathSegment::Index(3),
                PathSegment::Field("name".to_string()),
            ],
            code: "length_min".to_string(),
            message: "must be at least 1 character".to_string(),
            params: HashMap::new(),
        };
        assert_eq!(
            error.to_string(),
            "users[3].name: must be at least 1 character (length_min)"
        );
    }

    #[test]
    fn test_validation_error_display_without_path() {
        let error = ValidationError::new("custom", "cross-field validation failed");
        assert_eq!(error.to_string(), "cross-field validation failed (custom)");
    }

    #[test]
    fn test_path_string_empty() {
        let error = ValidationError::new("test", "test");
        assert_eq!(error.path_string(), "");
    }

    #[test]
    fn test_path_string_nested() {
        let error = ValidationError {
            path: vec![
                PathSegment::Field("a".to_string()),
                PathSegment::Field("b".to_string()),
                PathSegment::Index(0),
                PathSegment::Field("c".to_string()),
            ],
            code: "test".to_string(),
            message: "test".to_string(),
            params: HashMap::new(),
        };
        assert_eq!(error.path_string(), "a.b[0].c");
    }

    #[test]
    fn test_validation_errors_empty() {
        let errors = ValidationErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_validation_errors_add_and_len() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("a", "error a"));
        errors.add(ValidationError::new("b", "error b"));
        assert_eq!(errors.len(), 2);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validation_errors_merge() {
        let mut errors1 = ValidationErrors::new();
        errors1.add(ValidationError::new("a", "error a"));

        let mut errors2 = ValidationErrors::new();
        errors2.add(ValidationError::new("b", "error b"));
        errors2.add(ValidationError::new("c", "error c"));

        errors1.merge(errors2);
        assert_eq!(errors1.len(), 3);
    }

    #[test]
    fn test_validation_errors_field_errors() {
        let mut errors = ValidationErrors::new();
        errors.add(
            ValidationError::new("email", "invalid")
                .with_path(vec![PathSegment::Field("email".to_string())]),
        );
        errors.add(
            ValidationError::new("length", "too short")
                .with_path(vec![PathSegment::Field("name".to_string())]),
        );
        errors.add(
            ValidationError::new("email", "duplicate")
                .with_path(vec![PathSegment::Field("email".to_string())]),
        );

        let email_errors = errors.field_errors("email");
        assert_eq!(email_errors.len(), 2);

        let name_errors = errors.field_errors("name");
        assert_eq!(name_errors.len(), 1);

        let unknown_errors = errors.field_errors("unknown");
        assert_eq!(unknown_errors.len(), 0);
    }

    #[test]
    fn test_validation_errors_json_serialization() {
        let mut errors = ValidationErrors::new();
        errors.add(
            ValidationError::new("length_min", "must be at least 3 characters")
                .with_path(vec![PathSegment::Field("username".to_string())])
                .with_param("min", serde_json::json!(3))
                .with_param("actual", serde_json::json!(1)),
        );

        let json = serde_json::to_string(&errors).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let first_error = &parsed["errors"][0];
        assert_eq!(first_error["code"], "length_min");
        assert_eq!(first_error["path"][0], "username");
        assert_eq!(first_error["params"]["min"], 3);
    }

    #[test]
    fn test_validation_errors_display() {
        let mut errors = ValidationErrors::new();
        errors.add(
            ValidationError::new("email", "invalid email format")
                .with_path(vec![PathSegment::Field("email".to_string())]),
        );
        let display = errors.to_string();
        assert!(display.contains("Validation failed with 1 error(s)"));
        assert!(display.contains("email: invalid email format (email)"));
    }

    #[test]
    fn test_validation_errors_into_iterator() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("a", "A"));
        errors.add(ValidationError::new("b", "B"));

        let codes: Vec<String> = errors.into_iter().map(|e| e.code).collect();
        assert_eq!(codes, vec!["a", "b"]);
    }

    #[test]
    fn test_validation_error_with_param() {
        let error = ValidationError::new("length_min", "too short")
            .with_param("min", serde_json::json!(3))
            .with_param("actual", serde_json::json!(1));

        assert_eq!(error.params.len(), 2);
        assert_eq!(error.params["min"], serde_json::json!(3));
        assert_eq!(error.params["actual"], serde_json::json!(1));
    }

    #[test]
    fn test_params_skip_serializing_if_empty() {
        let error = ValidationError::new("email", "invalid");
        let json = serde_json::to_string(&error).unwrap();
        // params should not appear in JSON when empty
        assert!(!json.contains("params"));
    }
}
