//! Intermediate representation (IR) for validated structs.
//!
//! This module defines the domain model that sits between attribute parsing
//! (parse.rs) and code generation (codegen/). It represents the semantic
//! meaning of a Rusdantic-annotated struct, stripped of syntactic details.

use proc_macro2::Span;
use syn::{Expr, Generics, Ident, Path, Type};

/// A struct annotated with `#[derive(Rusdantic)]`, ready for code generation.
#[derive(Debug)]
pub struct ValidatedStruct {
    /// The struct's identifier (e.g., `User`)
    pub ident: Ident,
    /// Generic parameters and bounds
    pub generics: Generics,
    /// All fields with their validation rules
    pub fields: Vec<ValidatedField>,
    /// Struct-level configuration and validators
    pub config: StructConfig,
}

/// Struct-level configuration extracted from `#[rusdantic(...)]` attributes.
#[derive(Debug)]
#[allow(dead_code)] // Some fields are reserved for future codegen phases
pub struct StructConfig {
    /// Optional struct-level custom validation function that receives `&Self`
    pub custom_validator: Option<Path>,
    /// Serde rename_all strategy (e.g., "camelCase", "snake_case")
    pub rename_all: Option<RenameAll>,
    /// Whether to reject unknown fields during deserialization
    pub deny_unknown_fields: bool,
    /// Coercion mode: "strict" (default) or "lax"
    pub coerce_mode: CoerceMode,
    /// Whether to generate a custom Debug impl for redacted fields
    pub has_redacted_fields: bool,
}

/// Supported rename_all strategies, matching serde's conventions.
#[derive(Debug, Clone, Copy)]
pub enum RenameAll {
    /// `lowercase`
    Lowercase,
    /// `UPPERCASE`
    Uppercase,
    /// `camelCase`
    CamelCase,
    /// `PascalCase`
    PascalCase,
    /// `snake_case`
    SnakeCase,
    /// `SCREAMING_SNAKE_CASE`
    ScreamingSnakeCase,
    /// `kebab-case`
    KebabCase,
    /// `SCREAMING-KEBAB-CASE`
    ScreamingKebabCase,
}

impl RenameAll {
    /// Parse a rename_all string into a RenameAll variant.
    /// Returns None for unrecognized strategies.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(Self::Lowercase),
            "UPPERCASE" => Some(Self::Uppercase),
            "camelCase" => Some(Self::CamelCase),
            "PascalCase" => Some(Self::PascalCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(Self::ScreamingKebabCase),
            _ => None,
        }
    }

    /// Apply the rename strategy to a field name.
    pub fn apply(&self, name: &str) -> String {
        match self {
            Self::Lowercase => name.to_lowercase(),
            Self::Uppercase => name.to_uppercase(),
            Self::CamelCase => to_camel_case(name),
            Self::PascalCase => to_pascal_case(name),
            Self::SnakeCase => name.to_string(), // Rust fields are already snake_case
            Self::ScreamingSnakeCase => name.to_uppercase(),
            Self::KebabCase => name.replace('_', "-"),
            Self::ScreamingKebabCase => name.to_uppercase().replace('_', "-"),
        }
    }
}

/// Convert a snake_case string to camelCase.
fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            // First character stays lowercase in camelCase
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Convert a snake_case string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Coercion mode for type conversion during deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoerceMode {
    /// Exact type matching only (default). Rejects `"123"` for an integer field.
    #[default]
    Strict,
    /// Lax coercion: allows string-to-number, string-to-bool, etc.
    Lax,
}

/// A single struct field with its validation rules and metadata.
#[derive(Debug)]
#[allow(dead_code)] // Some fields are reserved for future codegen phases
pub struct ValidatedField {
    /// Field identifier (e.g., `username`)
    pub ident: Ident,
    /// Rust type of the field
    pub ty: Type,
    /// The primary serialized name for this field (used in serialization output,
    /// error paths, and JSON Schema). Determined by:
    /// serialization_alias > alias > rename_all > rust field name
    pub serialized_name: String,
    /// The primary deserialization name (used to match JSON keys during parsing).
    /// Determined by: validation_alias > alias > rename_all > rust field name.
    /// When an alias is set, BOTH the alias and the original name are accepted.
    pub deserialization_key: String,
    /// Additional names accepted during deserialization. When an alias is set,
    /// the Rust field name (after rename_all) is also accepted.
    pub deserialization_aliases: Vec<String>,
    /// Whether this field's type is `Option<T>` (validators skip on None)
    pub is_option: bool,
    /// Whether this field's type is a collection (`Vec<T>`, `HashSet<T>`, etc.)
    pub is_collection: bool,
    /// Validation rules to apply to this field
    pub rules: Vec<ValidationRule>,
    /// Sanitizers to apply before validation (during deserialization only)
    pub sanitizers: Vec<Sanitizer>,
    /// PII redaction configuration for Debug output
    pub redact: Option<RedactMode>,
    /// Deprecation metadata
    pub deprecated: Option<String>,
    /// Whether this is a computed field (serialization only)
    pub computed_method: Option<String>,
    /// Whether to apply type coercion for this field
    pub coerce: bool,
    /// Whether this field has the `nested` attribute for recursive validation
    pub nested: bool,
    /// The span of the field for error reporting
    pub span: Span,
}

/// A validation rule to be applied to a field value.
#[derive(Debug)]
pub enum ValidationRule {
    /// Minimum/maximum length for strings and collections.
    Length {
        min: Option<usize>,
        max: Option<usize>,
    },
    /// Minimum/maximum numeric value.
    Range {
        min: Option<Expr>,
        max: Option<Expr>,
    },
    /// Must be a valid email address.
    Email,
    /// Must be a valid URL.
    Url,
    /// Must match a regex pattern. The string is the raw regex.
    Pattern(String),
    /// Must contain a specific substring.
    Contains(String),
    /// Option<T> must be Some (not None).
    Required,
    /// Custom validation function path.
    Custom(Path),
    /// Recursively validate a nested struct.
    Nested,
}

/// A sanitizer transformation applied to a field value before validation.
#[derive(Debug)]
pub enum Sanitizer {
    /// Strip leading/trailing whitespace
    Trim,
    /// Convert to lowercase
    Lowercase,
    /// Convert to uppercase
    Uppercase,
    /// Truncate to a maximum number of characters
    Truncate(usize),
    /// Custom sanitizer function
    Custom(Path),
}

/// PII redaction mode for Debug/Display output.
#[derive(Debug)]
pub enum RedactMode {
    /// Replace with "[REDACTED]"
    Default,
    /// Replace with a custom string
    Custom(String),
    /// Show a truncated hash of the value
    Hash,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_all_camel_case() {
        assert_eq!(RenameAll::CamelCase.apply("user_name"), "userName");
        assert_eq!(RenameAll::CamelCase.apply("id"), "id");
        assert_eq!(
            RenameAll::CamelCase.apply("first_name_last"),
            "firstNameLast"
        );
    }

    #[test]
    fn test_rename_all_pascal_case() {
        assert_eq!(RenameAll::PascalCase.apply("user_name"), "UserName");
        assert_eq!(RenameAll::PascalCase.apply("id"), "Id");
    }

    #[test]
    fn test_rename_all_kebab_case() {
        assert_eq!(RenameAll::KebabCase.apply("user_name"), "user-name");
        assert_eq!(
            RenameAll::ScreamingKebabCase.apply("user_name"),
            "USER-NAME"
        );
    }

    #[test]
    fn test_rename_all_screaming_snake() {
        assert_eq!(
            RenameAll::ScreamingSnakeCase.apply("user_name"),
            "USER_NAME"
        );
    }

    #[test]
    fn test_coerce_mode_default() {
        assert_eq!(CoerceMode::default(), CoerceMode::Strict);
    }
}
