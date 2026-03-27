//! JSON Schema generation example — produce OpenAPI-compatible schemas.
//!
//! Run with: `cargo run --example json_schema`

use rusdantic::prelude::*;

#[derive(Rusdantic, Debug)]
struct CreateUserRequest {
    #[rusdantic(length(min = 3, max = 50))]
    username: String,

    #[rusdantic(email)]
    email: String,

    #[rusdantic(length(min = 8))]
    password: String,

    #[rusdantic(range(min = 13))]
    age: u8,

    #[rusdantic(url)]
    website: Option<String>,

    #[rusdantic(length(min = 1, max = 5))]
    roles: Vec<String>,

    #[rusdantic(pattern(regex = r"^[a-z]{2}$"))]
    country_code: String,
}

fn main() {
    let schema = CreateUserRequest::json_schema();

    println!("JSON Schema for CreateUserRequest:");
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    println!("\n=== Schema Details ===\n");

    // The schema includes:
    // - type: "object"
    // - properties with types and constraints
    // - required fields list
    // - format hints (email, uri)
    // - pattern for regex-validated fields
    // - minLength/maxLength for string constraints

    let props = schema["properties"].as_object().unwrap();
    for (name, prop) in props {
        let type_val = prop.get("type").and_then(|t| t.as_str()).unwrap_or("any");
        let constraints: Vec<String> = prop
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, _)| *k != "type")
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        if constraints.is_empty() {
            println!("  {}: {}", name, type_val);
        } else {
            println!("  {}: {} ({})", name, type_val, constraints.join(", "));
        }
    }

    println!("\n  Required: {:?}",
        schema["required"].as_array().unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
    );
}
