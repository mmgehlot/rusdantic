//! Custom validator example.
//!
//! Run with: `cargo run --example custom_validator`

use rusdantic::prelude::*;

/// Custom field-level validator: reject reserved usernames.
fn validate_not_reserved(value: &String) -> Result<(), ValidationError> {
    const RESERVED: &[&str] = &["admin", "root", "system", "moderator", "null", "undefined"];

    if RESERVED.contains(&value.to_lowercase().as_str()) {
        Err(
            ValidationError::new("reserved", format!("'{}' is a reserved username", value))
                .with_param("reserved_names", serde_json::json!(RESERVED)),
        )
    } else {
        Ok(())
    }
}

/// Custom struct-level validator: cross-field validation.
fn validate_password_match(value: &Registration) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    if value.password != value.password_confirm {
        errors.add(
            ValidationError::new(
                "password_mismatch",
                "password and password_confirm must match",
            )
            .with_path(vec![PathSegment::Field("password_confirm".to_string())]),
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
    #[rusdantic(
        length(min = 3, max = 20),
        custom(function = validate_not_reserved)
    )]
    username: String,

    #[rusdantic(email)]
    email: String,

    #[rusdantic(length(min = 8))]
    password: String,

    #[rusdantic(length(min = 8))]
    password_confirm: String,
}

fn main() {
    // Test with reserved username and mismatched passwords
    let json = r#"{
        "username": "admin",
        "email": "admin@example.com",
        "password": "secure123",
        "password_confirm": "different"
    }"#;

    match rusdantic::from_json::<Registration>(json) {
        Ok(reg) => println!("Registration: {:?}", reg),
        Err(e) => println!("Errors:\n{}", e),
    }

    // Test with valid data
    let json = r#"{
        "username": "alice",
        "email": "alice@example.com",
        "password": "secure123",
        "password_confirm": "secure123"
    }"#;

    match rusdantic::from_json::<Registration>(json) {
        Ok(reg) => println!("Valid registration: {:?}", reg),
        Err(e) => println!("Errors:\n{}", e),
    }
}
