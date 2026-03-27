//! Field aliases and rename_all example.
//!
//! Run with: `cargo run --example aliases_and_rename`

use rusdantic::prelude::*;

/// rename_all converts all field names to camelCase in JSON.
/// Individual fields can override with alias/serialization_alias.
#[derive(Rusdantic, Debug)]
#[rusdantic(rename_all = "camelCase")]
struct ApiResponse {
    #[rusdantic(length(min = 1))]
    first_name: String,  // JSON: "firstName"

    #[rusdantic(length(min = 1))]
    last_name: String,   // JSON: "lastName"

    /// Override rename_all: accepts "USERNAME" in JSON input,
    /// but serializes as "userName" (camelCase from rename_all)
    #[rusdantic(alias = "USERNAME")]
    user_name: String,

    /// Separate aliases for input and output:
    /// - Deserialization accepts: "user_email" or "userEmail" (rename_all)
    /// - Serialization outputs: "contactEmail"
    #[rusdantic(
        validation_alias = "user_email",
        serialization_alias = "contactEmail",
        email
    )]
    email_address: String,
}

fn main() {
    println!("=== Deserialize with camelCase ===\n");

    let json = r#"{
        "firstName": "Alice",
        "lastName": "Smith",
        "USERNAME": "alice_s",
        "user_email": "alice@example.com"
    }"#;

    match rusdantic::from_json::<ApiResponse>(json) {
        Ok(resp) => {
            println!("Parsed: {:?}", resp);

            // Serialize back — uses serialization names
            let output = serde_json::to_string_pretty(&resp).unwrap();
            println!("\nSerialized output:\n{}", output);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Error paths use alias names ===\n");

    let json = r#"{
        "firstName": "",
        "lastName": "Smith",
        "USERNAME": "alice",
        "user_email": "alice@example.com"
    }"#;

    match rusdantic::from_json::<ApiResponse>(json) {
        Ok(_) => println!("Unexpected pass"),
        Err(e) => println!("Error path shows 'firstName' (not 'first_name'):\n{}", e),
    }
}
