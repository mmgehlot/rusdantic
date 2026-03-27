//! Tests for validator modes and context injection.

use rusdantic::prelude::*;

// =============================================================================
// Validator modes
// =============================================================================

// After validator (default) — receives typed value
fn check_no_profanity(value: &String) -> Result<(), ValidationError> {
    let bad_words = ["badword", "spam"];
    for word in &bad_words {
        if value.to_lowercase().contains(word) {
            return Err(ValidationError::new(
                "profanity",
                format!("contains prohibited word: {}", word),
            ));
        }
    }
    Ok(())
}

#[derive(Rusdantic, Debug)]
struct WithAfterValidator {
    #[rusdantic(custom(function = check_no_profanity, mode = "after"))]
    username: String,
}

#[test]
fn test_after_validator_passes() {
    let data = WithAfterValidator {
        username: "good_user".to_string(),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_after_validator_fails() {
    let data = WithAfterValidator {
        username: "badword_user".to_string(),
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.errors()[0].code, "profanity");
}

// Default mode (no mode specified) is "after"
#[derive(Rusdantic, Debug)]
struct WithDefaultMode {
    #[rusdantic(custom(function = check_no_profanity))]
    text: String,
}

#[test]
fn test_default_mode_is_after() {
    let data = WithDefaultMode {
        text: "spam content".to_string(),
    };
    assert!(data.validate().is_err());
}

// =============================================================================
// Custom + built-in validators combined
// =============================================================================

#[derive(Rusdantic, Debug)]
struct CustomAndBuiltin {
    #[rusdantic(length(min = 3, max = 20), custom(function = check_no_profanity))]
    username: String,
}

#[test]
fn test_custom_and_builtin_combined() {
    let data = CustomAndBuiltin {
        username: "alice".to_string(),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_custom_and_builtin_both_fail() {
    let data = CustomAndBuiltin {
        username: "sp".to_string(), // too short (but no profanity)
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].code, "length_min");
}

// =============================================================================
// Struct-level cross-field validator
// =============================================================================

fn validate_date_order(value: &DateRange) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    if value.start > value.end {
        errors.add(
            ValidationError::new("date_order", "start must be before end")
                .with_path(vec![PathSegment::Field("end".to_string())]),
        );
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Rusdantic, Debug, Clone)]
#[rusdantic(custom(function = validate_date_order))]
struct DateRange {
    start: i64,
    end: i64,
}

#[test]
fn test_struct_level_validator_passes() {
    let data = DateRange {
        start: 100,
        end: 200,
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_struct_level_validator_fails() {
    let data = DateRange {
        start: 200,
        end: 100,
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.errors()[0].code, "date_order");
}

#[test]
fn test_struct_level_validator_works_via_from_json() {
    let json = r#"{"start": 200, "end": 100}"#;
    let result: Result<DateRange, _> = rusdantic::from_json(json);
    assert!(result.is_err()); // Cross-field validation also runs during deserialization
}

// =============================================================================
// Validator with both field-level and struct-level
// =============================================================================

fn validate_password_match(value: &Registration) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    if value.password != value.confirm {
        errors.add(
            ValidationError::new("mismatch", "passwords do not match")
                .with_path(vec![PathSegment::Field("confirm".to_string())]),
        );
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Rusdantic, Debug, Clone)]
#[rusdantic(custom(function = validate_password_match))]
struct Registration {
    #[rusdantic(email)]
    email: String,

    #[rusdantic(length(min = 8))]
    password: String,

    #[rusdantic(length(min = 8))]
    confirm: String,
}

#[test]
fn test_combined_field_and_struct_validators() {
    let data = Registration {
        email: "bad".to_string(),     // fails email
        password: "short".to_string(), // fails length
        confirm: "short2".to_string(), // fails length + mismatch
    };
    let err = data.validate().unwrap_err();
    // email (1) + password length (1) + confirm length (1) + mismatch (1) = 4
    assert!(err.len() >= 3);
}

#[test]
fn test_valid_registration() {
    let data = Registration {
        email: "user@example.com".to_string(),
        password: "securepass123".to_string(),
        confirm: "securepass123".to_string(),
    };
    assert!(data.validate().is_ok());
}
