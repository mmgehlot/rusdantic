# Contributing to Rusdantic

Thank you for your interest in contributing to Rusdantic! This document provides
guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust stable toolchain (MSRV: 1.70)
- `cargo` and `rustfmt` installed via `rustup`

### Development Setup

```bash
# Clone the repository
git clone https://github.com/mmgehlot/rusdantic.git
cd rusdantic

# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run clippy lints
cargo clippy --workspace --all-targets --all-features -- -Dwarnings

# Check formatting
cargo fmt --all --check
```

## Project Structure

```
rusdantic/              Facade crate (what users depend on)
rusdantic-core/         Validation traits, error types, built-in rules
rusdantic-derive/       Proc macro crate (#[derive(Rusdantic)])
```

- **rusdantic-core**: Contains the `Validate` trait, `ValidationError` types,
  and all built-in validator implementations. No proc-macro dependency.
- **rusdantic-derive**: The procedural macro crate. Generates `Serialize`,
  `Deserialize`, and `Validate` implementations from `#[derive(Rusdantic)]`.
- **rusdantic**: The facade crate that re-exports everything. This is what
  users add to their `Cargo.toml`.

## Making Changes

1. Fork the repository and create a feature branch from `main`.
2. Write tests before or alongside your changes.
3. Ensure all tests pass: `cargo test --workspace`
4. Ensure clippy is clean: `cargo clippy --workspace --all-targets -- -Dwarnings`
5. Ensure formatting is correct: `cargo fmt --all`
6. Submit a pull request with a clear description of the changes.

## Testing

### Unit Tests

Each crate has its own unit tests in `src/` files via `#[cfg(test)]` modules.

### Integration Tests

End-to-end tests live in `rusdantic/tests/`. These test the full derive macro
pipeline from attribute parsing through code generation and runtime validation.

### Compile-Fail Tests

We use `trybuild` for compile-fail tests in `rusdantic/tests/ui/`. These verify
that invalid attribute usage produces helpful compiler errors.

To update expected stderr output:
```bash
TRYBUILD=overwrite cargo test --test compile_tests
```

### Property Tests

We use `proptest` for property-based testing of validator implementations
in `rusdantic-core`.

## Code Style

- Follow `rustfmt` defaults (configured in `rustfmt.toml`)
- Use `///` doc comments on all public items
- Prefer explicit error handling over `unwrap()`/`expect()` in library code
- Use `syn::Error::new_spanned` for proc macro errors to ensure correct spans

## License

By contributing, you agree that your contributions will be dual-licensed under
MIT and Apache-2.0, matching the project's existing license.
