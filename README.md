# Rusdantic

[![Crates.io](https://img.shields.io/crates/v/rusdantic.svg)](https://crates.io/crates/rusdantic)
[![Documentation](https://docs.rs/rusdantic/badge.svg)](https://docs.rs/rusdantic)
[![CI](https://github.com/mmgehlot/rusdantic/actions/workflows/ci.yml/badge.svg)](https://github.com/mmgehlot/rusdantic/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/rusdantic.svg)](https://github.com/mmgehlot/rusdantic#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-blue.svg)](https://github.com/mmgehlot/rusdantic)

**Rusdantic** is a high-ergonomics data validation and serialization framework for Rust, inspired by Python's [Pydantic](https://docs.pydantic.dev). It bridges the gap between [Serde](https://serde.rs) and validation crates like [validator](https://crates.io/crates/validator) / [garde](https://crates.io/crates/garde) into a single, unified derive macro.

## Why Rusdantic?

In the Rust ecosystem, validation is fragmented across multiple crates. **Rusdantic** unifies serialization, deserialization, and validation into one derive macro:

- **One Derive, Three Traits**: `#[derive(Rusdantic)]` generates `Serialize`, `Deserialize`, and `Validate`
- **Validate-on-Deserialize**: Invalid structs never exist in memory when using `from_json()`
- **Path-Aware Errors**: Get precise error paths like `user.addresses[0].zip_code`
- **7 Built-in Validators**: `length`, `range`, `email`, `url`, `pattern`, `contains`, `required`
- **Custom Validators**: Field-level and struct-level cross-field validation
- **Serde Compatible**: Works with `rename_all`, `deny_unknown_fields`, and other serde attributes
- **JSON Schema**: Generate Draft 2020-12 / OpenAPI 3.1 schemas from your types
- **PII Redaction**: `#[rusdantic(redact)]` hides sensitive data in Debug output
- **Sanitizers**: `trim`, `lowercase`, `uppercase`, `truncate` during deserialization
- **Partial Validation**: Validate subsets of fields for PATCH endpoints
- **Zero-Cost**: Validation logic is monomorphized at compile time

## Quick Start

Add `rusdantic` to your `Cargo.toml`:

```toml
[dependencies]
rusdantic = "0.1.0"
```

### Basic Usage

```rust
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
    // Deserialize + validate in one step
    let json = r#"{"username": "rust_ace", "email": "user@example.com", "age": 25}"#;
    let user: User = rusdantic::from_json(json).unwrap();
    println!("Valid user: {:?}", user);

    // Invalid data returns all errors at once
    let bad_json = r#"{"username": "ab", "email": "bad", "age": 16}"#;
    match rusdantic::from_json::<User>(bad_json) {
        Ok(_) => unreachable!(),
        Err(e) => println!("{}", e),
        // Output:
        // username: must be at least 3 characters (length_min)
        // email: invalid email format (email)
        // age: must be at least 18 (range_min)
    }
}
```

### Nested Structs with Path-Aware Errors

```rust
#[derive(Rusdantic, Debug, Clone)]
struct Address {
    #[rusdantic(length(min = 1))]
    street: String,

    #[rusdantic(pattern(regex = r"^\d{5}$"))]
    zip_code: String,
}

#[derive(Rusdantic, Debug)]
struct UserProfile {
    #[rusdantic(length(min = 1))]
    name: String,

    #[rusdantic(nested)]
    addresses: Vec<Address>,
}

// Errors include full paths: "addresses[1].zip_code: must match pattern..."
```

### Custom Validators

```rust
fn validate_not_reserved(value: &String) -> Result<(), ValidationError> {
    if ["admin", "root"].contains(&value.as_str()) {
        Err(ValidationError::new("reserved", "username is reserved"))
    } else {
        Ok(())
    }
}

// Cross-field validation
fn validate_date_range(value: &Event) -> Result<(), ValidationErrors> {
    // Access all fields of the struct for cross-field checks
    // ...
}

#[derive(Rusdantic, Debug, Clone)]
#[rusdantic(custom(function = validate_date_range))]
struct Event {
    #[rusdantic(custom(function = validate_not_reserved))]
    organizer: String,
    start_date: String,
    end_date: String,
}
```

### Sanitizers

```rust
#[derive(Rusdantic, Debug)]
struct Registration {
    #[rusdantic(trim, lowercase, email)]
    email: String,  // "  User@EXAMPLE.COM  " → "user@example.com"

    #[rusdantic(trim, length(min = 3))]
    username: String,  // "  ab  " → "ab" → fails length(min=3)

    #[rusdantic(truncate(max = 100))]
    bio: String,
}
```

### PII Redaction

```rust
#[derive(Rusdantic)]
struct UserData {
    name: String,

    #[rusdantic(redact)]
    email: String,              // Debug shows: [REDACTED]

    #[rusdantic(redact(with = "***"))]
    ssn: String,                // Debug shows: ***

    #[rusdantic(redact(hash))]
    api_key: String,            // Debug shows: [HASH:a1b2c3d4...]
}
```

### JSON Schema Generation

```rust
let schema = User::json_schema();
// Produces:
// {
//   "$schema": "https://json-schema.org/draft/2020-12/schema",
//   "title": "User",
//   "type": "object",
//   "properties": {
//     "username": { "type": "string", "minLength": 3, "maxLength": 20 },
//     "email": { "type": "string", "format": "email" },
//     "age": { "type": "integer" }
//   },
//   "required": ["username", "email", "age"]
// }
```

### Partial Validation (PATCH endpoints)

```rust
let user = User { /* ... */ };

// Only validate specific fields
user.validate_partial(&["username", "email"])?;
```

## Feature Comparison

| Feature | serde + validator | serde + garde | Rusdantic |
|---|---|---|---|
| Setup | 3+ crates | 2+ crates | 1 crate |
| Validate on deser | No (invalid structs in memory) | No | Yes |
| Error paths | Manual (`serde_path_to_error`) | Built-in | Built-in |
| Type coercion | `serde_with` per-field | No | Configurable |
| Sanitizers | No | No | Built-in |
| JSON Schema | Separate (`schemars`) | No | Built-in |
| PII Redaction | No | No | Built-in |
| Partial validation | No | No | Built-in |

## Available Validators

| Attribute | Description | Example |
|---|---|---|
| `length(min, max)` | String/collection length | `#[rusdantic(length(min = 1, max = 255))]` |
| `range(min, max)` | Numeric bounds | `#[rusdantic(range(min = 0, max = 100))]` |
| `email` | Email format | `#[rusdantic(email)]` |
| `url` | URL format | `#[rusdantic(url)]` |
| `pattern(regex)` | Regex match | `#[rusdantic(pattern(regex = "^[a-z]+$"))]` |
| `contains(value)` | Substring check | `#[rusdantic(contains(value = "@"))]` |
| `required` | Option must be Some | `#[rusdantic(required)]` |
| `custom(function)` | Custom validator | `#[rusdantic(custom(function = my_fn))]` |
| `nested` | Recursive validation | `#[rusdantic(nested)]` |

## Available Sanitizers

| Attribute | Description |
|---|---|
| `trim` | Strip leading/trailing whitespace |
| `lowercase` | Convert to lowercase |
| `uppercase` | Convert to uppercase |
| `truncate(max = N)` | Truncate to N characters |

## Struct-Level Attributes

| Attribute | Description |
|---|---|
| `rename_all = "camelCase"` | Rename all fields (serde-compatible) |
| `deny_unknown_fields` | Reject unknown JSON keys |
| `custom(function = fn)` | Cross-field validation |

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

*Inspired by the ergonomics of [Pydantic](https://docs.pydantic.dev), built for the safety of Rust.*
