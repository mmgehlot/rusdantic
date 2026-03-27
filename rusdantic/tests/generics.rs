//! Tests for generic struct support.

use rusdantic::prelude::*;

/// Generic wrapper struct — Rusdantic generates Validate + Serialize.
/// Deserialize must be derived from serde separately for generic structs.
#[derive(Rusdantic, Debug, Clone, serde::Deserialize)]
struct GenWrapper<T: std::fmt::Debug + Clone> {
    #[rusdantic(length(min = 1))]
    label: String,
    inner: T,
}

#[test]
fn test_generic_validate_passes() {
    let w = GenWrapper {
        label: "test".to_string(),
        inner: 42i32,
    };
    assert!(w.validate().is_ok());
}

#[test]
fn test_generic_validate_fails() {
    let w = GenWrapper {
        label: "".to_string(),
        inner: 42i32,
    };
    let err = w.validate().unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].code, "length_min");
}

#[test]
fn test_generic_serialize() {
    let w = GenWrapper {
        label: "test".to_string(),
        inner: "hello".to_string(),
    };
    let json = serde_json::to_value(&w).unwrap();
    assert_eq!(json["label"], "test");
    assert_eq!(json["inner"], "hello");
}

#[test]
fn test_generic_deserialize_then_validate() {
    // For generic structs, use serde's Deserialize + Rusdantic's validate()
    let json = r#"{"label": "test", "inner": 99}"#;
    let w: GenWrapper<i32> = serde_json::from_str(json).unwrap();
    assert!(w.validate().is_ok());
    assert_eq!(w.label, "test");
    assert_eq!(w.inner, 99);
}

#[test]
fn test_generic_deserialize_invalid_then_validate() {
    let json = r#"{"label": "", "inner": 99}"#;
    let w: GenWrapper<i32> = serde_json::from_str(json).unwrap();
    let err = w.validate().unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].code, "length_min");
}

/// Generic paginated response
#[derive(Rusdantic, Debug, Clone, serde::Deserialize)]
struct Paginated<T: std::fmt::Debug + Clone> {
    items: Vec<T>,

    #[rusdantic(range(min = 1))]
    page: u32,

    #[rusdantic(range(min = 1, max = 100))]
    per_page: u32,
}

#[test]
fn test_generic_paginated_valid() {
    let p = Paginated {
        items: vec!["a".to_string(), "b".to_string()],
        page: 1,
        per_page: 10,
    };
    assert!(p.validate().is_ok());
}

#[test]
fn test_generic_paginated_invalid() {
    let p = Paginated {
        items: vec![1, 2, 3],
        page: 0,       // invalid: min = 1
        per_page: 200, // invalid: max = 100
    };
    let err = p.validate().unwrap_err();
    assert_eq!(err.len(), 2);
}

/// Generic with multiple type params
#[derive(Rusdantic, Debug, Clone, serde::Deserialize)]
struct Pair<A: std::fmt::Debug + Clone, B: std::fmt::Debug + Clone> {
    #[rusdantic(length(min = 1))]
    key: String,
    first: A,
    second: B,
}

#[test]
fn test_multi_generic_validate() {
    let p = Pair {
        key: "test".to_string(),
        first: 1i32,
        second: "hello".to_string(),
    };
    assert!(p.validate().is_ok());
}

#[test]
fn test_multi_generic_serialize() {
    let p = Pair {
        key: "test".to_string(),
        first: 1i32,
        second: true,
    };
    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(json["key"], "test");
    assert_eq!(json["first"], 1);
    assert_eq!(json["second"], true);
}
