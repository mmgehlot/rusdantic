//! Attribute parsing for `#[rusdantic(...)]` attributes.
//!
//! Uses `darling` for top-level struct/field extraction and custom parsing
//! for the validation rule DSL. This separation gives us darling's ergonomics
//! for structural parsing while retaining full control over the rule syntax.

use darling::ast::Data;
use darling::{FromDeriveInput, FromField, FromMeta};
use syn::{Expr, Ident, Path, Type};

/// Top-level parsed representation of a struct with `#[derive(Rusdantic)]`.
/// Darling automatically extracts the struct name, generics, visibility,
/// and iterates over fields to parse their attributes.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(rusdantic), supports(struct_named))]
pub struct RusdanticInput {
    /// The struct identifier (e.g., `User`)
    pub ident: Ident,
    /// Generic parameters on the struct
    pub generics: syn::Generics,
    /// Parsed fields with their rusdantic attributes
    pub data: Data<(), RusdanticField>,

    // --- Struct-level attributes ---
    /// Optional struct-level custom validation function path
    #[darling(default)]
    pub custom: Option<CustomValidator>,
    /// Serde-compatible rename_all strategy (e.g., "camelCase")
    #[darling(default)]
    pub rename_all: Option<String>,
    /// Whether to reject unknown fields during deserialization
    #[darling(default)]
    pub deny_unknown_fields: bool,
    /// Lax coercion mode: if true, strings can be coerced to numbers, etc.
    #[darling(default)]
    pub coerce_mode: Option<String>,
}

/// Parsed representation of a single struct field with its rusdantic attributes.
#[derive(Debug, FromField)]
#[darling(attributes(rusdantic))]
pub struct RusdanticField {
    /// The field identifier (e.g., `username`)
    pub ident: Option<Ident>,
    /// The field's Rust type (e.g., `String`, `Option<u8>`)
    pub ty: Type,

    // --- Field-level validation attributes ---
    /// Length constraint: `#[rusdantic(length(min = 3, max = 20))]`
    #[darling(default)]
    pub length: Option<LengthValidator>,
    /// Numeric range constraint: `#[rusdantic(range(min = 0, max = 100))]`
    #[darling(default)]
    pub range: Option<RangeValidator>,
    /// Email format validation: `#[rusdantic(email)]`
    #[darling(default)]
    pub email: bool,
    /// URL format validation: `#[rusdantic(url)]`
    #[darling(default)]
    pub url: bool,
    /// Regex pattern validation: `#[rusdantic(pattern(regex = "^[a-z]+$"))]`
    #[darling(default)]
    pub pattern: Option<PatternValidator>,
    /// String contains check: `#[rusdantic(contains(value = "@"))]`
    #[darling(default)]
    pub contains: Option<ContainsValidator>,
    /// Marks an Option<T> field as required (None is invalid):
    /// `#[rusdantic(required)]`
    #[darling(default)]
    pub required: bool,
    /// Custom validation function: `#[rusdantic(custom(function = my_fn))]`
    #[darling(default)]
    pub custom: Option<CustomValidator>,
    /// Recursively validate nested struct: `#[rusdantic(nested)]`
    #[darling(default)]
    pub nested: bool,

    // --- Alias attributes ---
    /// Field alias for deserialization: `#[rusdantic(alias = "userName")]`
    /// Accepts both the Rust field name and the alias during deserialization.
    #[darling(default)]
    pub alias: Option<String>,
    /// Alias used only for deserialization: `#[rusdantic(validation_alias = "user_name")]`
    #[darling(default)]
    pub validation_alias: Option<String>,
    /// Alias used only for serialization: `#[rusdantic(serialization_alias = "userName")]`
    #[darling(default)]
    pub serialization_alias: Option<String>,

    // --- Sanitizer attributes ---
    /// Trim whitespace: `#[rusdantic(trim)]`
    #[darling(default)]
    pub trim: bool,
    /// Convert to lowercase: `#[rusdantic(lowercase)]`
    #[darling(default)]
    pub lowercase: bool,
    /// Convert to uppercase: `#[rusdantic(uppercase)]`
    #[darling(default)]
    pub uppercase: bool,
    /// Truncate to max length: `#[rusdantic(truncate(max = 100))]`
    #[darling(default)]
    pub truncate: Option<TruncateValidator>,
    /// Custom sanitizer function: `#[rusdantic(sanitize(function = my_sanitizer))]`
    #[darling(default)]
    pub sanitize: Option<SanitizeFunction>,

    // --- Coercion attributes ---
    /// Per-field coercion override: `#[rusdantic(coerce)]`
    #[darling(default)]
    pub coerce: bool,

    // --- Metadata attributes ---
    /// PII redaction in Debug output: `#[rusdantic(redact)]`
    #[darling(default)]
    pub redact: Option<RedactConfig>,
    /// Field deprecation warning: `#[rusdantic(deprecated(message = "..."))]`
    #[darling(default)]
    pub deprecated: Option<DeprecatedConfig>,
    /// Computed field (serialization only): `#[rusdantic(computed)]`
    #[darling(default)]
    pub computed: Option<String>,
}

/// Length validation parameters.
/// Supports both string length (characters) and collection size (elements).
#[derive(Debug, FromMeta)]
pub struct LengthValidator {
    /// Minimum length (inclusive). Optional — omit for no lower bound.
    #[darling(default)]
    pub min: Option<usize>,
    /// Maximum length (inclusive). Optional — omit for no upper bound.
    #[darling(default)]
    pub max: Option<usize>,
}

/// Numeric range validation parameters.
/// Works with any numeric type that implements `PartialOrd`.
#[derive(Debug, FromMeta)]
pub struct RangeValidator {
    /// Minimum value (inclusive). Parsed as a syn expression to support
    /// integer and float literals.
    #[darling(default)]
    pub min: Option<Expr>,
    /// Maximum value (inclusive).
    #[darling(default)]
    pub max: Option<Expr>,
}

/// Regex pattern validation parameters.
#[derive(Debug, FromMeta)]
pub struct PatternValidator {
    /// The regex pattern string. Validated at compile time for correctness.
    pub regex: String,
}

/// String contains validation parameters.
#[derive(Debug, FromMeta)]
pub struct ContainsValidator {
    /// The substring that must be present in the field value.
    pub value: String,
}

/// Custom validation function reference.
#[derive(Debug, FromMeta)]
pub struct CustomValidator {
    /// Path to the validation function.
    pub function: Path,
}

/// Custom sanitizer function reference.
#[derive(Debug, FromMeta)]
pub struct SanitizeFunction {
    /// Path to the sanitizer function.
    pub function: Path,
}

/// Truncation parameters for string fields.
#[derive(Debug, FromMeta)]
pub struct TruncateValidator {
    /// Maximum length to truncate to.
    pub max: usize,
}

/// PII redaction configuration for Debug/Display output.
#[derive(Debug)]
pub enum RedactConfig {
    /// Replace with "[REDACTED]"
    Default,
    /// Replace with a custom string
    Custom(String),
    /// Show a hash of the value for correlation
    Hash,
}

/// Manual FromMeta implementation for RedactConfig to support multiple forms:
/// - `#[rusdantic(redact)]` → RedactConfig::Default
/// - `#[rusdantic(redact(with = "***"))]` → RedactConfig::Custom("***")
/// - `#[rusdantic(redact(hash))]` → RedactConfig::Hash
impl FromMeta for RedactConfig {
    fn from_word() -> darling::Result<Self> {
        Ok(RedactConfig::Default)
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        // Parse the nested meta items to determine the redaction mode
        for item in items {
            match item {
                darling::ast::NestedMeta::Meta(meta) => {
                    if meta.path().is_ident("hash") {
                        return Ok(RedactConfig::Hash);
                    }
                }
                darling::ast::NestedMeta::Lit(syn::Lit::Str(s)) => {
                    return Ok(RedactConfig::Custom(s.value()));
                }
                _ => {}
            }
        }
        // If we have a "with" key-value pair, extract it
        #[derive(FromMeta)]
        struct RedactWith {
            #[darling(default)]
            with: Option<String>,
            #[darling(default)]
            hash: bool,
        }
        let parsed = RedactWith::from_list(items)?;
        if parsed.hash {
            Ok(RedactConfig::Hash)
        } else if let Some(replacement) = parsed.with {
            Ok(RedactConfig::Custom(replacement))
        } else {
            Ok(RedactConfig::Default)
        }
    }
}

/// Field deprecation configuration.
#[derive(Debug, FromMeta)]
pub struct DeprecatedConfig {
    /// Human-readable deprecation message.
    pub message: String,
}
