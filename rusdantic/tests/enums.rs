//! Tests for enum support.
//!
//! Rusdantic generates a Validate impl for enums. Serialize/Deserialize
//! are provided by serde's own derives with matching serde attributes.

use rusdantic::prelude::*;

// =============================================================================
// Externally tagged enum (serde default)
// =============================================================================

#[derive(Rusdantic, Debug, serde::Serialize, serde::Deserialize)]
enum Shape {
    Circle {
        #[rusdantic(range(min = 0.0))]
        radius: f64,
    },
    Rectangle {
        #[rusdantic(range(min = 0.0))]
        width: f64,
        #[rusdantic(range(min = 0.0))]
        height: f64,
    },
    Point,
}

#[test]
fn test_enum_valid_circle() {
    let shape = Shape::Circle { radius: 5.0 };
    assert!(shape.validate().is_ok());
}

#[test]
fn test_enum_invalid_circle() {
    let shape = Shape::Circle { radius: -1.0 };
    let err = shape.validate().unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].code, "range_min");
}

#[test]
fn test_enum_valid_rectangle() {
    let shape = Shape::Rectangle {
        width: 10.0,
        height: 5.0,
    };
    assert!(shape.validate().is_ok());
}

#[test]
fn test_enum_invalid_rectangle_both_fields() {
    let shape = Shape::Rectangle {
        width: -1.0,
        height: -2.0,
    };
    let err = shape.validate().unwrap_err();
    assert_eq!(err.len(), 2);
}

#[test]
fn test_enum_unit_variant_always_valid() {
    let shape = Shape::Point;
    assert!(shape.validate().is_ok());
}

#[test]
fn test_enum_serialize_deserialize_roundtrip() {
    let shape = Shape::Circle { radius: 5.0 };
    let json = serde_json::to_string(&shape).unwrap();
    let deserialized: Shape = serde_json::from_str(&json).unwrap();
    assert!(deserialized.validate().is_ok());
}

// =============================================================================
// Internally tagged enum
// =============================================================================

#[derive(Rusdantic, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum Event {
    #[serde(rename = "click")]
    Click {
        #[rusdantic(range(min = 0))]
        x: i32,
        #[rusdantic(range(min = 0))]
        y: i32,
    },
    #[serde(rename = "keypress")]
    Keypress {
        #[rusdantic(length(min = 1))]
        key: String,
    },
}

#[test]
fn test_internally_tagged_valid() {
    let json = r#"{"type": "click", "x": 10, "y": 20}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    assert!(event.validate().is_ok());
}

#[test]
fn test_internally_tagged_invalid() {
    let json = r#"{"type": "click", "x": -1, "y": -2}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    let err = event.validate().unwrap_err();
    assert_eq!(err.len(), 2);
}

#[test]
fn test_internally_tagged_keypress() {
    let json = r#"{"type": "keypress", "key": "a"}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    assert!(event.validate().is_ok());
}

#[test]
fn test_internally_tagged_keypress_invalid() {
    let json = r#"{"type": "keypress", "key": ""}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    assert!(event.validate().is_err());
}

// =============================================================================
// Untagged enum
// =============================================================================

#[derive(Rusdantic, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum StringOrInt {
    Str {
        #[rusdantic(length(min = 1))]
        value: String,
    },
    Int {
        #[rusdantic(range(min = 0))]
        value: i32,
    },
}

#[test]
fn test_untagged_string_valid() {
    let json = r#"{"value": "hello"}"#;
    let v: StringOrInt = serde_json::from_str(json).unwrap();
    assert!(v.validate().is_ok());
}

#[test]
fn test_untagged_int_valid() {
    let json = r#"{"value": 42}"#;
    let v: StringOrInt = serde_json::from_str(json).unwrap();
    assert!(v.validate().is_ok());
}

// =============================================================================
// Enum with email validation
// =============================================================================

#[derive(Rusdantic, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "contact_type")]
enum Contact {
    Email {
        #[rusdantic(email)]
        address: String,
    },
    Phone {
        #[rusdantic(pattern(regex = r"^\+?[0-9]{10,15}$"))]
        number: String,
    },
}

#[test]
fn test_enum_email_variant_valid() {
    let c = Contact::Email {
        address: "user@example.com".to_string(),
    };
    assert!(c.validate().is_ok());
}

#[test]
fn test_enum_email_variant_invalid() {
    let c = Contact::Email {
        address: "not-email".to_string(),
    };
    assert!(c.validate().is_err());
}

#[test]
fn test_enum_phone_variant_valid() {
    let c = Contact::Phone {
        number: "+1234567890".to_string(),
    };
    assert!(c.validate().is_ok());
}

#[test]
fn test_enum_phone_variant_invalid() {
    let c = Contact::Phone {
        number: "abc".to_string(),
    };
    assert!(c.validate().is_err());
}
