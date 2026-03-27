//! Partial validation example — validate only specific fields.
//! Useful for PATCH endpoints where only some fields are updated.
//!
//! Run with: `cargo run --example partial_validation`

use rusdantic::prelude::*;

#[derive(Rusdantic, Debug)]
struct UserProfile {
    #[rusdantic(length(min = 3, max = 50))]
    name: String,

    #[rusdantic(email)]
    email: String,

    #[rusdantic(length(max = 500))]
    bio: String,

    #[rusdantic(range(min = 13, max = 120))]
    age: u8,
}

fn main() {
    // Imagine this struct came from a database with existing valid data,
    // then a PATCH request updates only "name" and "bio"
    let user = UserProfile {
        name: "ab".to_string(),      // INVALID: too short (min 3)
        email: "not-email".to_string(), // INVALID: bad format
        bio: "Hello world".to_string(), // valid
        age: 25,                         // valid
    };

    println!("=== Full validation (catches all errors) ===\n");
    match user.validate() {
        Ok(()) => println!("All valid"),
        Err(e) => println!("{}", e),
    }

    println!("=== Partial: validate only 'name' and 'bio' ===\n");
    match user.validate_partial(&["name", "bio"]) {
        Ok(()) => println!("name + bio are valid"),
        Err(e) => println!("{}", e),
    }

    println!("=== Partial: validate only 'age' ===\n");
    match user.validate_partial(&["age"]) {
        Ok(()) => println!("age is valid!"),
        Err(e) => println!("{}", e),
    }

    println!("=== Partial: unknown field name (catches typos) ===\n");
    match user.validate_partial(&["naem"]) {
        Ok(()) => println!("Unexpected pass"),
        Err(e) => println!("{}", e),
    }
}
