//! Enum validation example — validate fields inside enum variants.
//!
//! Run with: `cargo run --example enums`

use rusdantic::prelude::*;

/// Internally tagged enum — the "type" field determines the variant.
/// Use serde's tag attribute alongside Rusdantic's validation.
#[derive(Rusdantic, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum Notification {
    /// Email notification with validated address
    #[serde(rename = "email")]
    Email {
        #[rusdantic(email)]
        to: String,

        #[rusdantic(length(min = 1, max = 200))]
        subject: String,
    },

    /// SMS notification with validated phone number
    #[serde(rename = "sms")]
    Sms {
        #[rusdantic(pattern(regex = r"^\+[0-9]{10,15}$"))]
        phone: String,

        #[rusdantic(length(max = 160))]
        body: String,
    },

    /// Push notification with validated payload
    #[serde(rename = "push")]
    Push {
        #[rusdantic(length(min = 1))]
        device_token: String,

        #[rusdantic(length(max = 4096))]
        payload: String,
    },
}

fn main() {
    println!("=== Valid Notifications ===\n");

    let notifications = vec![
        r#"{"type": "email", "to": "user@example.com", "subject": "Welcome!"}"#,
        r#"{"type": "sms", "phone": "+1234567890", "body": "Your code is 1234"}"#,
        r#"{"type": "push", "device_token": "abc123", "payload": "{\"alert\": \"New message\"}"}"#,
    ];

    for json in &notifications {
        let notif: Notification = serde_json::from_str(json).unwrap();
        // Validate after deserialization (enum validation is separate)
        match notif.validate() {
            Ok(()) => println!("Valid: {:?}", notif),
            Err(e) => println!("Invalid: {}", e),
        }
    }

    println!("\n=== Invalid Notifications ===\n");

    let invalid = vec![
        r#"{"type": "email", "to": "not-an-email", "subject": ""}"#,
        r#"{"type": "sms", "phone": "invalid", "body": "test"}"#,
    ];

    for json in &invalid {
        let notif: Notification = serde_json::from_str(json).unwrap();
        match notif.validate() {
            Ok(()) => println!("Unexpected pass: {:?}", notif),
            Err(e) => println!("Caught: {}", e),
        }
    }
}
