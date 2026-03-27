# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release of Rusdantic
- `#[derive(Rusdantic)]` macro generating `Serialize`, `Deserialize`, and `Validate` implementations
- Built-in validators: `length`, `range`, `email`, `url`, `pattern`, `contains`, `required`
- Custom field-level and struct-level validators
- Path-aware error reporting with nested struct and collection support
- Validate-on-deserialize: validation embedded in `Deserialize` impl
- Serde attribute compatibility: `rename`, `rename_all`, `default`, `skip`, `alias`, `deny_unknown_fields`
- Type coercion with `lax` and `strict` modes
- Field sanitizers: `trim`, `lowercase`, `uppercase`, `truncate`, custom
- JSON Schema generation (Draft 2020-12 / OpenAPI 3.1)
- Computed fields for serialization-only values
- Context injection for validators requiring external state
- PII redaction in Debug/Display output via `#[rusdantic(redact)]`
- Compile-time configuration validation (min <= max, valid regex, etc.)
- Partial validation for PATCH endpoints
- Field deprecation warnings
- WebAssembly support
