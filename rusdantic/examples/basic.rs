//! Basic usage example for Rusdantic.
//!
//! Run with: `cargo run --example basic`

use rusdantic::prelude::*;

#[derive(Rusdantic, Debug)]
struct User {
    #[rusdantic(length(min = 3, max = 20))]
    username: String,

    #[rusdantic(email)]
    email: String,

    #[rusdantic(range(min = 18))]
    age: u8,
}

fn main() {
    // Example 1: Valid JSON → successful deserialization + validation
    let valid_json = r#"{
        "username": "rust_ace",
        "email": "user@example.com",
        "age": 25
    }"#;

    match rusdantic::from_json::<User>(valid_json) {
        Ok(user) => println!("Valid user: {:?}", user),
        Err(e) => println!("Error: {}", e),
    }

    // Example 2: Invalid data → validation errors with paths
    let invalid_json = r#"{
        "username": "ab",
        "email": "not-an-email",
        "age": 16
    }"#;

    match rusdantic::from_json::<User>(invalid_json) {
        Ok(user) => println!("User: {:?}", user),
        Err(e) => println!("Errors:\n{}", e),
    }

    // Example 3: Post-construction validation
    let user = User {
        username: "valid_name".to_string(),
        email: "user@example.com".to_string(),
        age: 25,
    };

    match user.validate() {
        Ok(()) => println!("Validation passed!"),
        Err(errors) => {
            for error in errors.errors() {
                println!(
                    "  {}: {} ({})",
                    error.path_string(),
                    error.message,
                    error.code
                );
            }
        }
    }

    // Example 4: JSON Schema generation
    let schema = User::json_schema();
    println!(
        "\nJSON Schema:\n{}",
        serde_json::to_string_pretty(&schema).unwrap()
    );
}
