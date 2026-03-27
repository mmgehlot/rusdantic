//! Tests for field sanitizer support.
//! Sanitizers transform field values during deserialization, before validation.

use rusdantic::prelude::*;

// =============================================================================
// Trim sanitizer
// =============================================================================

#[derive(Rusdantic, Debug)]
struct TrimStruct {
    #[rusdantic(trim, length(min = 1))]
    name: String,
}

#[test]
fn test_trim_removes_whitespace() {
    let json = r#"{"name": "  hello  "}"#;
    let result: TrimStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.name, "hello");
}

#[test]
fn test_trim_then_validate_empty_fails() {
    // "   " trimmed becomes "", which fails length(min=1)
    let json = r#"{"name": "   "}"#;
    let result: Result<TrimStruct, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

// =============================================================================
// Lowercase sanitizer
// =============================================================================

#[derive(Rusdantic, Debug)]
struct LowercaseStruct {
    #[rusdantic(lowercase)]
    email: String,
}

#[test]
fn test_lowercase_converts() {
    let json = r#"{"email": "USER@EXAMPLE.COM"}"#;
    let result: LowercaseStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.email, "user@example.com");
}

// =============================================================================
// Uppercase sanitizer
// =============================================================================

#[derive(Rusdantic, Debug)]
struct UppercaseStruct {
    #[rusdantic(uppercase)]
    code: String,
}

#[test]
fn test_uppercase_converts() {
    let json = r#"{"code": "abc123"}"#;
    let result: UppercaseStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.code, "ABC123");
}

// =============================================================================
// Truncate sanitizer
// =============================================================================

#[derive(Rusdantic, Debug)]
struct TruncateStruct {
    #[rusdantic(truncate(max = 5))]
    short_name: String,
}

#[test]
fn test_truncate_long_string() {
    let json = r#"{"short_name": "hello world"}"#;
    let result: TruncateStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.short_name, "hello");
}

#[test]
fn test_truncate_short_string_unchanged() {
    let json = r#"{"short_name": "hi"}"#;
    let result: TruncateStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.short_name, "hi");
}

// =============================================================================
// Combined sanitizers: trim + lowercase
// =============================================================================

#[derive(Rusdantic, Debug)]
struct CombinedSanitizers {
    #[rusdantic(trim, lowercase, email)]
    email: String,
}

#[test]
fn test_combined_trim_lowercase() {
    let json = r#"{"email": "  User@Example.COM  "}"#;
    let result: CombinedSanitizers = rusdantic::from_json(json).unwrap();
    assert_eq!(result.email, "user@example.com");
}

// =============================================================================
// Sanitizers only apply during deserialization, not .validate()
// =============================================================================

#[test]
fn test_sanitizers_do_not_apply_on_validate() {
    // When constructing directly and calling .validate(),
    // sanitizers do NOT run — the value is used as-is
    let data = TrimStruct {
        name: "  hello  ".to_string(),
    };
    // Still passes because the raw value "  hello  " has length > 1
    assert!(data.validate().is_ok());
}
