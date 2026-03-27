//! Advanced serialization with DumpOptions — include/exclude fields,
//! skip null values, pretty-print with custom indent.
//!
//! Run with: `cargo run --example dump_options`

use rusdantic::prelude::*;

#[derive(Rusdantic, Debug)]
struct UserData {
    name: String,
    email: String,
    password_hash: String,
    bio: Option<String>,
    age: Option<u8>,
    internal_id: String,
}

fn main() {
    let user = UserData {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: "$2b$12$hashedvalue".to_string(),
        bio: None,
        age: Some(30),
        internal_id: "int-12345".to_string(),
    };

    println!("=== Default dump (all fields) ===\n");
    println!("{}", user.dump_json().unwrap());

    println!("\n=== Exclude sensitive fields ===\n");
    let opts = DumpOptions::new().exclude(&["password_hash", "internal_id"]);
    let json = user.dump_json_with(&opts).unwrap();
    println!("{}", json);

    println!("\n=== Include only public fields ===\n");
    let opts = DumpOptions::new().include(&["name", "email", "bio", "age"]);
    let json = user.dump_json_with(&opts).unwrap();
    println!("{}", json);

    println!("\n=== Exclude null values ===\n");
    let opts = DumpOptions::new()
        .exclude(&["password_hash", "internal_id"])
        .exclude_none(true);
    let json = user.dump_json_with(&opts).unwrap();
    println!("{}", json);

    println!("\n=== Pretty-printed with 4-space indent ===\n");
    let opts = DumpOptions::new()
        .exclude(&["password_hash", "internal_id"])
        .exclude_none(true)
        .indent(4);
    let json = user.dump_json_with(&opts).unwrap();
    println!("{}", json);
}
