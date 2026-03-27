//! Tests for type coercion (lax mode).
//!
//! In lax mode, Rusdantic accepts values that can be reasonably converted
//! to the target type (e.g., "123" → 123, 1 → true).

use rusdantic::prelude::*;

// =============================================================================
// Struct-level coerce_mode = "lax"
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(coerce_mode = "lax")]
struct LaxStruct {
    count: i32,
    ratio: f64,
    active: bool,
    label: String,
}

#[test]
fn test_lax_string_to_int() {
    let json = r#"{"count": "42", "ratio": 1.5, "active": true, "label": "test"}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.count, 42);
}

#[test]
fn test_lax_string_to_float() {
    let json = r#"{"count": 1, "ratio": "3.15", "active": true, "label": "test"}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert!((result.ratio - 3.15).abs() < f64::EPSILON);
}

#[test]
fn test_lax_string_to_bool() {
    let json = r#"{"count": 1, "ratio": 1.0, "active": "true", "label": "test"}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert!(result.active);
}

#[test]
fn test_lax_int_to_bool() {
    let json = r#"{"count": 1, "ratio": 1.0, "active": 1, "label": "test"}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert!(result.active);
}

#[test]
fn test_lax_number_to_string() {
    let json = r#"{"count": 1, "ratio": 1.0, "active": true, "label": 42}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.label, "42");
}

#[test]
fn test_lax_bool_to_string() {
    let json = r#"{"count": 1, "ratio": 1.0, "active": true, "label": false}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.label, "false");
}

#[test]
fn test_lax_float_to_int_no_fraction() {
    let json = r#"{"count": 42.0, "ratio": 1.0, "active": true, "label": "test"}"#;
    let result: LaxStruct = rusdantic::from_json(json).unwrap();
    assert_eq!(result.count, 42);
}

#[test]
fn test_lax_float_to_int_with_fraction_fails() {
    let json = r#"{"count": 42.5, "ratio": 1.0, "active": true, "label": "test"}"#;
    let result: Result<LaxStruct, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_lax_invalid_string_to_int_fails() {
    let json = r#"{"count": "abc", "ratio": 1.0, "active": true, "label": "test"}"#;
    let result: Result<LaxStruct, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_lax_invalid_int_to_bool_fails() {
    // Only 0 and 1 can be coerced to bool
    let json = r#"{"count": 1, "ratio": 1.0, "active": 2, "label": "test"}"#;
    let result: Result<LaxStruct, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

// =============================================================================
// Per-field coerce attribute
// =============================================================================

#[derive(Rusdantic, Debug)]
struct PerFieldCoerce {
    // This field accepts string → int coercion
    #[rusdantic(coerce)]
    flexible_count: i32,

    // This field uses strict mode (default)
    strict_count: i32,
}

#[test]
fn test_per_field_coerce_flexible() {
    let json = r#"{"flexible_count": "99", "strict_count": 42}"#;
    let result: PerFieldCoerce = rusdantic::from_json(json).unwrap();
    assert_eq!(result.flexible_count, 99);
    assert_eq!(result.strict_count, 42);
}

#[test]
fn test_per_field_strict_rejects_string() {
    let json = r#"{"flexible_count": 1, "strict_count": "42"}"#;
    let result: Result<PerFieldCoerce, _> = rusdantic::from_json(json);
    assert!(result.is_err()); // strict field rejects string
}

// =============================================================================
// Coercion with validation
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(coerce_mode = "lax")]
struct CoerceWithValidation {
    #[rusdantic(range(min = 18))]
    age: u8,

    #[rusdantic(email)]
    email: String,
}

#[test]
fn test_coerce_then_validate_passes() {
    let json = r#"{"age": "25", "email": "user@example.com"}"#;
    let result: CoerceWithValidation = rusdantic::from_json(json).unwrap();
    assert_eq!(result.age, 25);
}

#[test]
fn test_coerce_then_validate_fails() {
    let json = r#"{"age": "16", "email": "not-email"}"#;
    let result: Result<CoerceWithValidation, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

// =============================================================================
// Coercion with various int sizes
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(coerce_mode = "lax")]
struct IntSizes {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: i8,
    f: i16,
    g: i32,
    h: i64,
}

#[test]
fn test_coerce_all_int_sizes() {
    let json = r#"{"a":"1","b":"2","c":"3","d":"4","e":"-1","f":"-2","g":"-3","h":"-4"}"#;
    let result: IntSizes = rusdantic::from_json(json).unwrap();
    assert_eq!(result.a, 1);
    assert_eq!(result.b, 2);
    assert_eq!(result.c, 3);
    assert_eq!(result.d, 4);
    assert_eq!(result.e, -1);
    assert_eq!(result.f, -2);
    assert_eq!(result.g, -3);
    assert_eq!(result.h, -4);
}

#[test]
fn test_coerce_overflow_rejected() {
    // "256" overflows u8
    let json = r#"{"a":"256","b":"2","c":"3","d":"4","e":"1","f":"2","g":"3","h":"4"}"#;
    let result: Result<IntSizes, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

// =============================================================================
// Default strict mode — coercion does NOT happen
// =============================================================================

#[derive(Rusdantic, Debug)]
struct StrictByDefault {
    count: i32,
}

#[test]
fn test_strict_rejects_string_by_default() {
    let json = r#"{"count": "42"}"#;
    let result: Result<StrictByDefault, _> = rusdantic::from_json(json);
    assert!(result.is_err()); // strict mode rejects string → int
}

#[test]
fn test_strict_accepts_correct_type() {
    let json = r#"{"count": 42}"#;
    let result: StrictByDefault = rusdantic::from_json(json).unwrap();
    assert_eq!(result.count, 42);
}

// =============================================================================
// Option<T> + coercion
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(coerce_mode = "lax")]
struct OptionalCoerce {
    #[rusdantic(range(min = 0))]
    score: Option<i32>,

    name: Option<String>,
}

#[test]
fn test_option_coerce_string_to_int() {
    let json = r#"{"score": "42", "name": "alice"}"#;
    let result: OptionalCoerce = rusdantic::from_json(json).unwrap();
    assert_eq!(result.score, Some(42));
    assert_eq!(result.name.as_deref(), Some("alice"));
}

#[test]
fn test_option_coerce_null_is_none() {
    let json = r#"{"score": null, "name": null}"#;
    let result: OptionalCoerce = rusdantic::from_json(json).unwrap();
    assert_eq!(result.score, None);
    assert_eq!(result.name, None);
}

#[test]
fn test_option_coerce_missing_is_none() {
    let json = r#"{}"#;
    let result: OptionalCoerce = rusdantic::from_json(json).unwrap();
    assert_eq!(result.score, None);
    assert_eq!(result.name, None);
}

#[test]
fn test_option_coerce_with_validation() {
    let json = r#"{"score": "-5"}"#;
    let result: Result<OptionalCoerce, _> = rusdantic::from_json(json);
    // "−5" coerces to -5, which fails range(min=0) validation
    assert!(result.is_err());
}
