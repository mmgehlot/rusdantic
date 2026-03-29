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
- **Custom Validators**: Field-level and struct-level cross-field validation with context injection
- **Type Coercion**: Lax mode accepts `"123"` as an integer, `"true"` as a bool
- **Field Aliases**: Separate names for deserialization input, serialization output, and error paths
- **Enum Support**: Validates fields inside enum variants (all serde representations)
- **Serde Compatible**: Works with `rename_all`, `deny_unknown_fields`, and other serde attributes
- **JSON Schema**: Generate Draft 2020-12 / OpenAPI 3.1 schemas from your types
- **PII Redaction**: `#[rusdantic(redact)]` hides sensitive data in Debug output with constant-time comparison
- **Sanitizers**: `trim`, `lowercase`, `uppercase`, `truncate` during deserialization
- **Partial Validation**: Validate subsets of fields for PATCH endpoints
- **Advanced Serialization**: `DumpOptions` with include/exclude, exclude_none, pretty-print
- **Constrained Types**: `PositiveInt`, `EmailStr`, `SecretStr`, `HttpUrl`, and more
- **Settings Management**: Load config from env vars, .env files, JSON with type coercion
- **Zero-Cost**: Validation logic is monomorphized at compile time

## Quick Start

Add `rusdantic` to your `Cargo.toml`:

```toml
[dependencies]
rusdantic = "0.1.0"
```

### Basic Validation

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

    // Generate JSON Schema
    let schema = User::json_schema();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

> Run this example: `cargo run --example basic`

## Examples

Rusdantic ships with **11 runnable examples** covering every major feature. Run any with `cargo run --example <name>`.

### Type Coercion

Lax mode automatically converts between compatible types during deserialization:

```rust
#[derive(Rusdantic, Debug)]
#[rusdantic(coerce_mode = "lax")]
struct ApiRequest {
    #[rusdantic(range(min = 0, max = 150))]
    age: u8,           // Accepts: 25, "25", 25.0
    active: bool,      // Accepts: true, "true", "yes", 1
    label: String,     // Accepts: "hello", 42, true
    score: Option<i32>, // Accepts: null, "42", 42
}

// Per-field coercion (rest stays strict):
#[derive(Rusdantic, Debug)]
struct MixedMode {
    #[rusdantic(coerce, range(min = 1))]
    flexible_id: i64,  // Accepts "123"
    count: i32,        // Rejects "123" (strict)
}
```

> Run this example: `cargo run --example coercion`

### Nested Structs with Path-Aware Errors

```rust
#[derive(Rusdantic, Debug, Clone)]
struct Address {
    #[rusdantic(length(min = 1))]
    street: String,
    #[rusdantic(pattern(regex = r"^\d{5}(-\d{4})?$"))]
    zip_code: String,
}

#[derive(Rusdantic, Debug)]
struct UserProfile {
    #[rusdantic(length(min = 1))]
    name: String,
    #[rusdantic(nested)]
    addresses: Vec<Address>,
}

// Error output includes full paths:
//   addresses[1].zip_code: must match pattern '^\d{5}(-\d{4})?$' (pattern)
//   addresses[1].street: must be at least 1 characters (length_min)
```

> Run this example: `cargo run --example nested`

### Field Aliases & Rename

```rust
#[derive(Rusdantic, Debug)]
#[rusdantic(rename_all = "camelCase")]
struct ApiResponse {
    #[rusdantic(length(min = 1))]
    first_name: String,  // JSON key: "firstName"

    // Override rename_all for this field
    #[rusdantic(alias = "USERNAME")]
    user_name: String,   // Accepts both "USERNAME" and "userName"

    // Separate input/output names
    #[rusdantic(
        validation_alias = "user_email",     // Accepts "user_email" in JSON input
        serialization_alias = "contactEmail", // Outputs as "contactEmail"
        email
    )]
    email_address: String,
}
```

> Run this example: `cargo run --example aliases_and_rename`

### Enum Validation

Validate fields inside enum variants with any serde representation:

```rust
#[derive(Rusdantic, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum Notification {
    #[serde(rename = "email")]
    Email {
        #[rusdantic(email)]
        to: String,
        #[rusdantic(length(min = 1, max = 200))]
        subject: String,
    },
    #[serde(rename = "sms")]
    Sms {
        #[rusdantic(pattern(regex = r"^\+[0-9]{10,15}$"))]
        phone: String,
        #[rusdantic(length(max = 160))]
        body: String,
    },
}

let notif: Notification = serde_json::from_str(json)?;
notif.validate()?;
```

> Run this example: `cargo run --example enums`

### Sanitizers & PII Redaction

Sanitizers transform values during deserialization, *before* validation runs:

```rust
#[derive(Rusdantic, Debug)]
struct Registration {
    #[rusdantic(trim, lowercase, length(min = 3, max = 20))]
    username: String,     // "  Alice_123  " -> "alice_123"

    #[rusdantic(trim, lowercase, email)]
    email: String,        // "  USER@Example.COM  " -> "user@example.com"

    #[rusdantic(truncate(max = 100))]
    bio: String,          // Truncated to 100 chars
}
```

PII redaction ensures sensitive data never appears in logs:

```rust
#[derive(Rusdantic)]
struct Secrets {
    name: String,

    #[rusdantic(redact)]
    email: String,                // Debug: [REDACTED]

    #[rusdantic(redact(with = "***-**-****"))]
    ssn: String,                  // Debug: ***-**-****

    #[rusdantic(redact(hash))]
    api_key: String,              // Debug: [HASH:a1b2c3d4e5f6...]
}
```

> Run this example: `cargo run --example sanitizers_and_redaction`

### Custom Validators & Cross-Field Validation

```rust
fn validate_not_reserved(value: &String) -> Result<(), ValidationError> {
    let reserved = ["admin", "root", "system"];
    if reserved.contains(&value.to_lowercase().as_str()) {
        Err(ValidationError::new("reserved", format!("'{}' is reserved", value)))
    } else {
        Ok(())
    }
}

fn validate_password_match(value: &Registration) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    if value.password != value.confirm {
        errors.add(ValidationError::new("mismatch", "passwords do not match")
            .with_path(vec![PathSegment::Field("confirm".to_string())]));
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[derive(Rusdantic, Debug, Clone)]
#[rusdantic(custom(function = validate_password_match))]
struct Registration {
    #[rusdantic(length(min = 3), custom(function = validate_not_reserved))]
    username: String,
    #[rusdantic(length(min = 8))]
    password: String,
    #[rusdantic(length(min = 8))]
    confirm: String,
}
```

> Run this example: `cargo run --example custom_validator`

### Advanced Serialization (Dump Options)

```rust
use rusdantic::prelude::*;

let user = UserData { /* ... */ };

// Exclude sensitive fields
let opts = DumpOptions::new()
    .exclude(&["password_hash", "internal_id"]);
let json = user.dump_json_with(&opts)?;

// Exclude null values + pretty print
let opts = DumpOptions::new()
    .exclude(&["password_hash"])
    .exclude_none(true)
    .indent(4);
let json = user.dump_json_with(&opts)?;

// Include only specific fields
let opts = DumpOptions::new()
    .include(&["name", "email"]);
let json = user.dump_json_with(&opts)?;
```

> Run this example: `cargo run --example dump_options`

### Partial Validation (PATCH Endpoints)

```rust
#[derive(Rusdantic, Debug)]
struct UserProfile {
    #[rusdantic(length(min = 3, max = 50))]
    name: String,
    #[rusdantic(email)]
    email: String,
    #[rusdantic(range(min = 13, max = 120))]
    age: u8,
}

let user = UserProfile { /* existing data from DB */ };

// Only validate the fields being updated by the PATCH request
user.validate_partial(&["name", "email"])?;

// Typos are caught:
user.validate_partial(&["naem"]); // Error: unknown field 'naem'
```

> Run this example: `cargo run --example partial_validation`

### Constrained Types

Pre-built types that validate at construction and deserialization time:

```rust
use rusdantic::types::*;

#[derive(serde::Deserialize)]
struct Config {
    port: PositiveInt<u16>,       // Must be > 0
    ratio: FiniteFloat<f64>,      // No NaN or Infinity
    name: NonEmptyString,         // Must have >= 1 character
    contact: EmailStr,            // Validated email format
    website: HttpUrl,             // Validated HTTP/HTTPS URL
    api_key: SecretStr,           // Redacted in Debug/Display/Serialize
}

// Types validate during deserialization:
let json = r#"{"port": 0}"#;  // Error: value must be positive (> 0)

// SecretStr never leaks:
let secret = SecretStr::new("sk-live-abc123");
println!("{:?}", secret);           // SecretStr("**********")
println!("{}", secret);             // **********
println!("{}", secret.expose_secret()); // sk-live-abc123
```

> Run this example: `cargo run --example types_library`

### JSON Schema Generation

```rust
let schema = CreateUserRequest::json_schema();

// Produces Draft 2020-12 / OpenAPI 3.1 compatible schema:
// {
//   "$schema": "https://json-schema.org/draft/2020-12/schema",
//   "title": "CreateUserRequest",
//   "type": "object",
//   "properties": {
//     "username": { "type": "string", "minLength": 3, "maxLength": 50 },
//     "email": { "type": "string", "format": "email" },
//     "age": { "type": "integer" },
//     "website": { "anyOf": [{"type": "string", "format": "uri"}, {"type": "null"}] },
//     "roles": { "type": "array", "minItems": 1, "maxItems": 5 },
//     "country_code": { "type": "string", "pattern": "^[a-z]{2}$" }
//   },
//   "required": ["username", "email", "age", "roles", "country_code"]
// }
```

> Run this example: `cargo run --example json_schema`

## Feature Comparison

| Feature | serde + validator | serde + garde | Rusdantic |
|---|---|---|---|
| Setup | 3+ crates | 2+ crates | 1 crate |
| Validate on deser | No (invalid structs in memory) | No | Yes |
| Error paths | Manual (`serde_path_to_error`) | Built-in | Built-in |
| Type coercion | `serde_with` per-field | No | Configurable (strict/lax) |
| Field aliases | `serde(alias)` only | No | alias + validation_alias + serialization_alias |
| Enum validation | Manual | Limited | Built-in |
| Sanitizers | No | No | trim, lowercase, uppercase, truncate |
| JSON Schema | Separate (`schemars`) | No | Built-in |
| PII Redaction | No | No | 3 modes (redact, custom, hash) |
| Partial validation | No | No | Built-in with typo detection |
| Secret types | No | No | SecretStr, SecretBytes, Secret\<T\> |
| Settings mgmt | No | No | Env, dotenv, JSON |
| Dump options | No | No | include/exclude/exclude_none |

## Available Validators

| Attribute | Description | Example |
|---|---|---|
| `length(min, max)` | String/collection length | `#[rusdantic(length(min = 1, max = 255))]` |
| `range(min, max)` | Numeric bounds (incl. i128/u128) | `#[rusdantic(range(min = 0, max = 100))]` |
| `email` | Email format | `#[rusdantic(email)]` |
| `url` | URL format | `#[rusdantic(url)]` |
| `pattern(regex)` | Regex match (auto-anchored) | `#[rusdantic(pattern(regex = "^[a-z]+$"))]` |
| `contains(value)` | Substring check | `#[rusdantic(contains(value = "@"))]` |
| `required` | Option must be Some | `#[rusdantic(required)]` |
| `custom(function)` | Custom validator | `#[rusdantic(custom(function = my_fn))]` |
| `custom_with_context(function)` | Context-aware validator | `#[rusdantic(custom_with_context(function = my_fn))]` |
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
| `rename_all = "camelCase"` | Rename all fields (8 strategies supported) |
| `deny_unknown_fields` | Reject unknown JSON keys |
| `coerce_mode = "lax"` | Enable type coercion for all fields |
| `custom(function = fn)` | Cross-field validation |

## Field-Level Attributes

| Attribute | Description |
|---|---|
| `alias = "name"` | Accept alternative name in JSON input + output |
| `validation_alias = "name"` | Accept alternative name in JSON input only |
| `serialization_alias = "name"` | Use alternative name in JSON output only |
| `coerce` | Enable type coercion for this field |
| `redact` / `redact(with = "...")` / `redact(hash)` | PII redaction in Debug |
| `deprecated(message = "...")` | Emit deprecation warning during deserialization |
| `computed = "method_name"` | Include method output in serialization only |

## Crate Ecosystem

| Crate | Description |
|---|---|
| [`rusdantic`](https://crates.io/crates/rusdantic) | Facade crate (re-exports everything) |
| [`rusdantic-core`](https://crates.io/crates/rusdantic-core) | Runtime: traits, errors, validators, coercion, dump |
| [`rusdantic-derive`](https://crates.io/crates/rusdantic-derive) | Proc macro: `#[derive(Rusdantic)]` |
| [`rusdantic-types`](https://crates.io/crates/rusdantic-types) | Constrained types: PositiveInt, EmailStr, SecretStr, HttpUrl |
| [`rusdantic-settings`](https://crates.io/crates/rusdantic-settings) | Settings management: env, dotenv, JSON |

## All Examples

Run any example with `cargo run --example <name>`:

| Example | Features Demonstrated |
|---|---|
| `basic` | Derive, from_json, validate, JSON Schema |
| `nested` | Nested structs, path-aware errors, collections |
| `custom_validator` | Field-level + struct-level cross-field validators |
| `coercion` | Lax mode, per-field coerce, Option+null, strict rejection |
| `enums` | Internally tagged enums, variant field validation |
| `sanitizers_and_redaction` | trim/lowercase/truncate + PII redaction modes |
| `aliases_and_rename` | rename_all, alias, validation_alias, serialization_alias |
| `dump_options` | include/exclude, exclude_none, pretty-print |
| `types_library` | PositiveInt, FiniteFloat, NonEmptyString, EmailStr, SecretStr, HttpUrl |
| `partial_validation` | PATCH endpoint partial validation, unknown field detection |
| `json_schema` | JSON Schema generation with constraints |

## Security Notes

- **PII `redact(hash)` mode**: Uses `std::collections::hash_map::DefaultHasher` for **correlation purposes only** — NOT cryptographically secure. For production secret hashing, use SHA-256 or bcrypt in a custom validator.
- **`SecretStr` Hash trait**: The `Hash` implementation is NOT constant-time. Avoid using `SecretStr` as a `HashMap` key in security-sensitive contexts.
- **Settings `from_dotenv()`**: Reads from a file path — ensure the path is trusted and not user-controlled to prevent path traversal.
- **Custom validators**: Must not panic. Panics during validation propagate through serde and may cause unexpected behavior.
- **Computed fields**: Methods called during serialization bypass validation and redaction. Do not return sensitive data from computed methods.

## Known Limitations

- **Generic structs**: `#[derive(Rusdantic)]` on generic types generates `Validate` + `Serialize` but NOT `Deserialize`. Add `#[derive(serde::Deserialize)]` manually and call `.validate()` after deserialization. `json_schema()` and `validate_partial()` are also unavailable for generic types.
- **Enum support**: Enums get only `Validate` impl — you must also derive `serde::Serialize` and `serde::Deserialize`. Tuple variants skip field validation.
- **Sanitizers**: Only applied during `from_json()` deserialization, NOT when calling `.validate()` on manually constructed structs.
- **`validate_partial()`**: Field names must use **serialized** names (e.g., `"camelCase"` if `rename_all = "camelCase"`), not Rust field names.

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

*Inspired by the ergonomics of [Pydantic](https://docs.pydantic.dev), built for the safety of Rust.*
