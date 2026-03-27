//! Sanitizers and PII redaction example.
//!
//! Run with: `cargo run --example sanitizers_and_redaction`

use rusdantic::prelude::*;

/// Sanitizers transform field values during deserialization, BEFORE validation.
/// This means " hello " becomes "hello" before length checks run.
#[derive(Rusdantic, Debug)]
struct UserRegistration {
    /// Trim whitespace and convert to lowercase before validation
    #[rusdantic(trim, lowercase, length(min = 3, max = 20))]
    username: String,

    /// Trim and lowercase email before format validation
    #[rusdantic(trim, lowercase, email)]
    email: String,

    /// Truncate bio to 100 characters
    #[rusdantic(truncate(max = 100))]
    bio: String,
}

/// PII redaction prevents sensitive data from appearing in Debug output.
/// This is critical for preventing accidental secret leakage in logs.
#[derive(Rusdantic)]
struct UserProfile {
    name: String,

    /// Shows [REDACTED] in Debug output
    #[rusdantic(redact)]
    email: String,

    /// Shows custom replacement in Debug output
    #[rusdantic(redact(with = "***-**-****"))]
    ssn: String,

    /// Shows a hash for log correlation without exposing the value
    #[rusdantic(redact(hash))]
    api_key: String,
}

fn main() {
    println!("=== Sanitizers ===\n");

    // Input with messy whitespace and mixed case
    let json = r#"{
        "username": "  Alice_123  ",
        "email": "  USER@Example.COM  ",
        "bio": "This is a really long bio that will be truncated to 100 characters. It keeps going and going and going and going until it exceeds the limit."
    }"#;

    match rusdantic::from_json::<UserRegistration>(json) {
        Ok(user) => {
            println!("Username: '{}' (trimmed + lowercased)", user.username);
            println!("Email: '{}' (trimmed + lowercased)", user.email);
            println!("Bio length: {} chars (truncated from original)", user.bio.len());
        }
        Err(e) => println!("Error: {}", e),
    }

    // Sanitizer + validation: "   ab   " trims to "ab" which fails length(min=3)
    let json = r#"{"username": "   ab   ", "email": "a@b.com", "bio": "test"}"#;
    match rusdantic::from_json::<UserRegistration>(json) {
        Ok(_) => println!("\nUnexpected pass"),
        Err(e) => println!("\nSanitize then validate: {}", e),
    }

    println!("\n=== PII Redaction ===\n");

    let profile = UserProfile {
        name: "Alice Smith".to_string(),
        email: "alice@secret.com".to_string(),
        ssn: "123-45-6789".to_string(),
        api_key: "sk-live-abc123def456".to_string(),
    };

    // Debug output is safe for logging — secrets are redacted
    println!("Debug output (safe for logs):");
    println!("{:?}", profile);
    println!();
    println!("Notice:");
    println!("  - name is visible (not redacted)");
    println!("  - email shows [REDACTED]");
    println!("  - ssn shows ***-**-****");
    println!("  - api_key shows [HASH:...] for correlation");
}
