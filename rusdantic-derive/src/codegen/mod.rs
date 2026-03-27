//! Code generation for `#[derive(Rusdantic)]`.
//!
//! This module orchestrates the generation of three trait implementations:
//! 1. `serde::Serialize` — standard serialization with rename support
//! 2. `serde::Deserialize` — deserialization with embedded validation
//! 3. `rusdantic_core::Validate` — post-construction validation
//!
//! Additionally generates:
//! 4. Custom `Debug` impl when any fields have `#[rusdantic(redact)]`
//! 5. `rusdantic_core::JsonSchema` impl for JSON Schema generation
//! 6. `validate_partial` method for partial validation

pub mod deserialize;
pub mod schema;
pub mod serialize;
pub mod validate;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::diagnostics;
use crate::model::{
    CoerceMode, RedactMode, RenameAll, Sanitizer, StructConfig, ValidatedField, ValidatedStruct,
    ValidationRule,
};
use crate::parse::{RedactConfig, RusdanticField, RusdanticInput};

/// Main entry point for the derive macro expansion.
/// Parses attributes, validates configuration, converts to IR, and generates code.
pub fn expand_rusdantic(input: DeriveInput) -> syn::Result<TokenStream> {
    // Step 1: Parse attributes using darling
    let parsed = RusdanticInput::from_derive_input(&input)?;

    // Step 2: Validate configurations at compile time (min<=max, valid regex, etc.)
    let diag_errors = diagnostics::validate_config(&parsed);
    if !diag_errors.is_empty() {
        // Combine all diagnostic errors into a single error for reporting
        let mut combined = diag_errors.into_iter();
        let first = combined.next().unwrap();
        let err = combined.fold(first, |mut acc, e| {
            acc.combine(e);
            acc
        });
        return Err(err);
    }

    // Step 3: Convert parsed attributes into the intermediate representation (IR)
    let validated = build_ir(parsed)?;

    // Step 4: Generate all trait implementations from the IR
    let validate_impl = validate::generate_validate_impl(&validated);
    let deserialize_impl = deserialize::generate_deserialize_impl(&validated);
    let serialize_impl = serialize::generate_serialize_impl(&validated);
    let debug_impl = if validated.config.has_redacted_fields {
        generate_debug_impl(&validated)
    } else {
        TokenStream::new()
    };
    let schema_impl = schema::generate_schema_impl(&validated);
    let partial_validate = validate::generate_partial_validate(&validated);

    // Step 5: Combine all generated code into a single token stream
    Ok(quote::quote! {
        #validate_impl
        #deserialize_impl
        #serialize_impl
        #debug_impl
        #schema_impl
        #partial_validate
    })
}

/// Convert the parsed darling representation into our intermediate representation.
/// This is where we resolve rename strategies, detect Option<T> types, etc.
fn build_ir(parsed: RusdanticInput) -> syn::Result<ValidatedStruct> {
    let rename_all = parsed
        .rename_all
        .as_deref()
        .and_then(RenameAll::from_str);

    let coerce_mode = match parsed.coerce_mode.as_deref() {
        Some("lax") => CoerceMode::Lax,
        _ => CoerceMode::Strict,
    };

    // Extract fields from the darling Data enum
    let darling_fields = match parsed.data {
        darling::ast::Data::Struct(fields) => fields.fields,
        _ => {
            return Err(syn::Error::new(
                parsed.ident.span(),
                "Rusdantic currently only supports named structs",
            ));
        }
    };

    // Convert each darling field into our IR representation
    let mut fields = Vec::with_capacity(darling_fields.len());
    let mut has_redacted_fields = false;

    for f in darling_fields {
        let field = convert_field(f, rename_all, coerce_mode)?;
        if field.redact.is_some() {
            has_redacted_fields = true;
        }
        fields.push(field);
    }

    Ok(ValidatedStruct {
        ident: parsed.ident,
        generics: parsed.generics,
        fields,
        config: StructConfig {
            custom_validator: parsed.custom.map(|c| c.function),
            rename_all,
            deny_unknown_fields: parsed.deny_unknown_fields,
            coerce_mode,
            has_redacted_fields,
        },
    })
}

/// Convert a single darling-parsed field into our IR field representation.
fn convert_field(
    field: RusdanticField,
    rename_all: Option<RenameAll>,
    struct_coerce: CoerceMode,
) -> syn::Result<ValidatedField> {
    let ident = field.ident.clone().ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Rusdantic only supports named fields",
        )
    })?;

    // Determine the serialized name by applying rename_all if present
    let raw_name = ident.to_string();
    let serialized_name = rename_all
        .map(|r| r.apply(&raw_name))
        .unwrap_or_else(|| raw_name.clone());

    // Detect if the type is Option<T>
    let is_option = is_option_type(&field.ty);

    // Detect if the type is a collection (Vec, HashSet, BTreeSet, etc.)
    let is_collection = is_collection_type(&field.ty);

    // Build validation rules from parsed attributes
    let mut rules = Vec::new();

    if let Some(length) = field.length {
        rules.push(ValidationRule::Length {
            min: length.min,
            max: length.max,
        });
    }
    if let Some(range) = field.range {
        rules.push(ValidationRule::Range {
            min: range.min,
            max: range.max,
        });
    }
    if field.email {
        rules.push(ValidationRule::Email);
    }
    if field.url {
        rules.push(ValidationRule::Url);
    }
    if let Some(pattern) = field.pattern {
        rules.push(ValidationRule::Pattern(pattern.regex));
    }
    if let Some(contains) = field.contains {
        rules.push(ValidationRule::Contains(contains.value));
    }
    if field.required {
        rules.push(ValidationRule::Required);
    }
    if let Some(custom) = field.custom {
        rules.push(ValidationRule::Custom(custom.function));
    }
    if field.nested {
        rules.push(ValidationRule::Nested);
    }

    // Build sanitizers list
    let mut sanitizers = Vec::new();
    if field.trim {
        sanitizers.push(Sanitizer::Trim);
    }
    if field.lowercase {
        sanitizers.push(Sanitizer::Lowercase);
    }
    if field.uppercase {
        sanitizers.push(Sanitizer::Uppercase);
    }
    if let Some(truncate) = field.truncate {
        sanitizers.push(Sanitizer::Truncate(truncate.max));
    }
    if let Some(sanitize) = field.sanitize {
        sanitizers.push(Sanitizer::Custom(sanitize.function));
    }

    // Convert redact config
    let redact = field.redact.map(|r| match r {
        RedactConfig::Default => RedactMode::Default,
        RedactConfig::Custom(s) => RedactMode::Custom(s),
        RedactConfig::Hash => RedactMode::Hash,
    });

    // Determine coercion: field-level override or inherit struct-level setting
    let coerce = field.coerce || struct_coerce == CoerceMode::Lax;

    Ok(ValidatedField {
        span: ident.span(),
        ident,
        ty: field.ty,
        serialized_name,
        is_option,
        is_collection,
        rules,
        sanitizers,
        redact,
        deprecated: field.deprecated.map(|d| d.message),
        computed_method: field.computed,
        coerce,
        nested: field.nested,
    })
}

/// Check if a type is `Option<T>` by inspecting the outermost path segment.
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Check if a type is a known collection type (Vec, HashSet, BTreeSet, VecDeque).
fn is_collection_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let name = segment.ident.to_string();
            return matches!(
                name.as_str(),
                "Vec" | "HashSet" | "BTreeSet" | "VecDeque" | "LinkedList"
            );
        }
    }
    false
}

/// Generate a custom `Debug` implementation that redacts sensitive fields.
fn generate_debug_impl(validated: &ValidatedStruct) -> TokenStream {
    let name = &validated.ident;
    let (impl_generics, ty_generics, where_clause) = validated.generics.split_for_impl();
    let name_str = name.to_string();

    // For each field, generate either the real debug output or a redacted version
    let field_debug_entries: Vec<TokenStream> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_none()) // Skip computed fields in Debug
        .map(|field| {
            let field_ident = &field.ident;
            let field_name = field_ident.to_string();

            match &field.redact {
                Some(RedactMode::Default) => {
                    quote::quote! {
                        .field(#field_name, &"[REDACTED]")
                    }
                }
                Some(RedactMode::Custom(replacement)) => {
                    quote::quote! {
                        .field(#field_name, &#replacement)
                    }
                }
                Some(RedactMode::Hash) => {
                    // Use a simple hash for correlation without exposing the value.
                    // We use DefaultHasher which is stable within a single program run.
                    quote::quote! {
                        .field(#field_name, &{
                            use std::hash::{Hash, Hasher};
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            self.#field_ident.hash(&mut hasher);
                            format!("[HASH:{:016x}]", hasher.finish())
                        })
                    }
                }
                None => {
                    // Normal debug output for non-redacted fields
                    quote::quote! {
                        .field(#field_name, &self.#field_ident)
                    }
                }
            }
        })
        .collect();

    quote::quote! {
        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(#name_str)
                    #(#field_debug_entries)*
                    .finish()
            }
        }
    }
}
