# Rusdantic 🦀
[![Crates.io](https://img.shields.io)](https://crates.io)
[![Documentation](https://docs.rs)](https://docs.rs)
[![License](https://img.shields.io)](LICENSE)

**Rusdantic** is a high-ergonomics data validation and serialization framework for Rust. It bridges the gap between the modular power of [Serde](https://serde.rs) and the developer-friendly, unified experience of Python's [Pydantic](https://docs.pydantic.dev).
## ✨ Why Rusdantic?
In the current Rust ecosystem, validation is often fragmented between `serde` for parsing and separate crates like `validator` for logic. **Rusdantic** unifies these into a single, intuitive workflow.
- **Unified Derive Macro**: Define serialization and validation rules in one place.
- **Path-Aware Errors**: Get precise error reports (e.g., `user.profiles[0].email: invalid format`).- **Zero-Cost Abstractions**: Built on top of Rust's powerful type system for maximum performance.
- **Type-Safe Validation**: Leverage Rust's `Option`, `Result`, and custom types to ensure data integrity at compile-time and runtime.
## 🚀 Quick Start
Add `rusdantic` to your `Cargo.toml`:

```toml
[dependencies]
rusdantic = "0.1.0"

Basic Usage

use rusdantic::{Rusdantic, Validate};

#[derive(Rusdantic, Debug)]struct User {
    #[rusdantic(length(min = 3, max = 20))]
    username: String,
    
    #[rusdantic(email)]
    email: String,
    
    #[rusdantic(range(min = 18))]
    age: u8,
}
fn main() {
    let json_data = r#"{
        "username": "rust_ace",
        "email": "invalid-email",
        "age": 16
    }"#;

    let result = User::from_json(json_data);

    match result {
        Ok(user) => println!("Validated User: {:?}", user),
        Err(e) => println!("Validation Errors: \n{}", e),
    }
}

🛠 Features vs. The "Standard" Stack

| Feature | Serde + Validator | Rusdantic |
|---|---|---|
| Setup | Multiple crates/traits | Single crate |
| Error Handling | Manual glue code | Automatic & Nested |
| Context | Limited | Full path awareness |
| DX | Verbose | Fluid & Python-like |

🤝 Contributing
We welcome contributions! Please see our CONTRIBUTING.md for details on how to get started.
📜 License
Licensed under either of:

* Apache License, Version 2.0 (LICENSE-APACHE)
* MIT license (LICENSE-MIT)

------------------------------
Inspired by the ergonomics of Pydantic, built for the safety of Rust.

