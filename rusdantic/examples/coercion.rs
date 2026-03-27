//! Type coercion example — lax mode accepts strings as numbers, bools, etc.
//!
//! Run with: `cargo run --example coercion`

use rusdantic::prelude::*;

/// In lax mode, fields accept multiple input types:
/// - Strings that parse as numbers: "42" → 42
/// - Strings that parse as bools: "true" → true
/// - Numbers that convert to strings: 42 → "42"
/// - Integers as bools: 1 → true, 0 → false
#[derive(Rusdantic, Debug)]
#[rusdantic(coerce_mode = "lax")]
struct ApiRequest {
    /// Accepts: 25, "25", 25.0
    #[rusdantic(range(min = 0, max = 150))]
    age: u8,

    /// Accepts: true, "true", "yes", 1
    active: bool,

    /// Accepts: "hello", 42, true (all converted to string)
    label: String,

    /// Optional coerced field: accepts null, "42", 42
    #[rusdantic(range(min = 0))]
    score: Option<i32>,
}

/// Per-field coercion: only specific fields are lax, rest are strict.
#[derive(Rusdantic, Debug)]
struct MixedMode {
    /// This field accepts "123" as an integer
    #[rusdantic(coerce, range(min = 1))]
    flexible_id: i64,

    /// This field ONLY accepts an actual integer (strict by default)
    count: i32,
}

fn main() {
    println!("=== Lax Mode (struct-level) ===\n");

    // All fields accept string representations
    let json = r#"{
        "age": "25",
        "active": "yes",
        "label": 42,
        "score": "100"
    }"#;

    match rusdantic::from_json::<ApiRequest>(json) {
        Ok(req) => println!("Parsed: {:?}", req),
        Err(e) => println!("Error: {}", e),
    }

    // Null for optional field
    let json = r#"{"age": 30, "active": true, "label": "test", "score": null}"#;
    match rusdantic::from_json::<ApiRequest>(json) {
        Ok(req) => println!("With null score: {:?}", req),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Mixed Mode (per-field) ===\n");

    // flexible_id accepts string, count requires integer
    let json = r#"{"flexible_id": "999", "count": 5}"#;
    match rusdantic::from_json::<MixedMode>(json) {
        Ok(req) => println!("Mixed: {:?}", req),
        Err(e) => println!("Error: {}", e),
    }

    // count rejects string (strict)
    let json = r#"{"flexible_id": "999", "count": "5"}"#;
    match rusdantic::from_json::<MixedMode>(json) {
        Ok(_) => println!("Should not reach here"),
        Err(e) => println!("Strict rejection: {}", e),
    }
}
