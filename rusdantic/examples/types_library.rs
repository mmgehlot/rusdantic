//! Constrained types example — types that validate at construction time.
//!
//! Run with: `cargo run --example types_library`

use rusdantic::types::*;

fn main() {
    println!("=== Numeric Types ===\n");

    // PositiveInt: must be > 0
    match PositiveInt::<i32>::new(42) {
        Ok(p) => println!("PositiveInt(42) = {}", *p),
        Err(e) => println!("Error: {}", e),
    }
    match PositiveInt::<i32>::new(0) {
        Ok(_) => println!("Should not reach"),
        Err(e) => println!("PositiveInt(0) rejected: {}", e),
    }

    // FiniteFloat: rejects NaN and Infinity
    match FiniteFloat::<f64>::new(3.14) {
        Ok(f) => println!("FiniteFloat(3.14) = {}", *f),
        Err(e) => println!("Error: {}", e),
    }
    match FiniteFloat::<f64>::new(f64::NAN) {
        Ok(_) => println!("Should not reach"),
        Err(e) => println!("FiniteFloat(NaN) rejected: {}", e),
    }

    println!("\n=== String Types ===\n");

    // NonEmptyString: must have at least 1 character
    match NonEmptyString::new("hello") {
        Ok(s) => println!("NonEmptyString = '{}'", &*s),
        Err(e) => println!("Error: {}", e),
    }
    match NonEmptyString::new("") {
        Ok(_) => println!("Should not reach"),
        Err(e) => println!("NonEmptyString('') rejected: {}", e),
    }

    // EmailStr: validated email format
    match EmailStr::new("user@example.com") {
        Ok(e) => println!("EmailStr = '{}'", &*e),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Secret Types ===\n");

    // SecretStr: redacted in Debug/Display, serializes as null
    let secret = SecretStr::new("sk-live-abc123");
    println!("Debug: {:?}", secret);
    println!("Display: {}", secret);
    println!("Actual value: {}", secret.expose_secret());
    println!(
        "Serialized: {}",
        serde_json::to_string(&secret).unwrap()
    );

    println!("\n=== Network Types ===\n");

    // HttpUrl: validated HTTP/HTTPS URL
    match HttpUrl::new("https://api.example.com/v1") {
        Ok(u) => println!("HttpUrl = '{}'", &*u),
        Err(e) => println!("Error: {}", e),
    }
    match HttpUrl::new("ftp://files.example.com") {
        Ok(_) => println!("Should not reach"),
        Err(e) => println!("HttpUrl(ftp://...) rejected: {}", e),
    }

    println!("\n=== Deserialization with Validation ===\n");

    // Types validate during deserialization
    let json = r#"{"port": 0}"#;
    match serde_json::from_str::<PortConfig>(json) {
        Ok(_) => println!("Should not reach"),
        Err(e) => println!("PositiveInt(0) in JSON rejected: {}", e),
    }

    let json = r#"{"port": 8080}"#;
    match serde_json::from_str::<PortConfig>(json) {
        Ok(config) => println!("Valid config: port = {}", *config.port),
        Err(e) => println!("Error: {}", e),
    }
}

#[derive(serde::Deserialize, Debug)]
struct PortConfig {
    port: PositiveInt<u16>,
}
