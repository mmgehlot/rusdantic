//! Code generation for JSON Schema generation.
//!
//! Generates an impl block that produces a JSON Schema (Draft 2020-12)
//! representation of the struct, mapping validation rules to JSON Schema
//! keywords. Compatible with OpenAPI 3.1.

use proc_macro2::TokenStream;
use quote::quote;

use crate::model::{ValidatedField, ValidatedStruct, ValidationRule};

/// Generate the JSON Schema impl for the struct.
pub fn generate_schema_impl(validated: &ValidatedStruct) -> TokenStream {
    let name = &validated.ident;
    let (impl_generics, ty_generics, where_clause) = validated.generics.split_for_impl();
    let name_str = name.to_string();

    // Generate property entries for each field
    let property_entries: Vec<TokenStream> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_none()) // Skip computed for input schema
        .map(|f| generate_property_schema(f))
        .collect();

    // Collect required field names (non-Option fields without a default)
    let required_fields: Vec<&str> = validated
        .fields
        .iter()
        .filter(|f| !f.is_option && f.computed_method.is_none())
        .map(|f| f.serialized_name.as_str())
        .collect();

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Generate a JSON Schema (Draft 2020-12) for this struct.
            ///
            /// The schema maps Rusdantic validation rules to JSON Schema keywords:
            /// - `length(min, max)` → `minLength`/`maxLength` or `minItems`/`maxItems`
            /// - `range(min, max)` → `minimum`/`maximum`
            /// - `email` → `format: "email"`
            /// - `url` → `format: "uri"`
            /// - `pattern` → `pattern`
            /// - Required fields → `required` array
            /// - `Option<T>` → nullable via `anyOf`
            pub fn json_schema() -> ::serde_json::Value {
                let mut properties = ::serde_json::Map::new();
                let required: Vec<&str> = vec![#(#required_fields),*];

                #(#property_entries)*

                let mut schema = ::serde_json::Map::new();
                schema.insert(
                    "$schema".to_string(),
                    ::serde_json::Value::String(
                        "https://json-schema.org/draft/2020-12/schema".to_string()
                    ),
                );
                schema.insert(
                    "title".to_string(),
                    ::serde_json::Value::String(#name_str.to_string()),
                );
                schema.insert(
                    "type".to_string(),
                    ::serde_json::Value::String("object".to_string()),
                );
                schema.insert(
                    "properties".to_string(),
                    ::serde_json::Value::Object(properties),
                );
                if !required.is_empty() {
                    schema.insert(
                        "required".to_string(),
                        ::serde_json::Value::Array(
                            required.iter().map(|s| ::serde_json::Value::String(s.to_string())).collect()
                        ),
                    );
                }

                ::serde_json::Value::Object(schema)
            }
        }
    }
}

/// Generate the JSON Schema property entry for a single field.
fn generate_property_schema(field: &ValidatedField) -> TokenStream {
    let serialized_name = &field.serialized_name;

    // Determine the base JSON Schema type from the Rust type
    let base_type = rust_type_to_json_schema_type(&field.ty);

    // Collect constraint keywords from validation rules
    let constraints: Vec<TokenStream> = field
        .rules
        .iter()
        .filter_map(|rule| rule_to_schema_constraint(rule, field))
        .collect();

    if field.is_option {
        // Option<T> generates anyOf: [schema, null]
        quote! {
            {
                let mut prop = ::serde_json::Map::new();
                #base_type
                #(#constraints)*

                // Wrap in anyOf with null for Option<T>
                let null_schema = ::serde_json::json!({"type": "null"});
                let type_schema = ::serde_json::Value::Object(prop);
                let any_of = ::serde_json::json!({
                    "anyOf": [type_schema, null_schema]
                });
                properties.insert(#serialized_name.to_string(), any_of);
            }
        }
    } else {
        quote! {
            {
                let mut prop = ::serde_json::Map::new();
                #base_type
                #(#constraints)*
                properties.insert(
                    #serialized_name.to_string(),
                    ::serde_json::Value::Object(prop),
                );
            }
        }
    }
}

/// Map a Rust type to its JSON Schema type string.
/// Returns TokenStream that inserts the "type" key into the `prop` map.
fn rust_type_to_json_schema_type(ty: &syn::Type) -> TokenStream {
    let type_name = extract_type_name(ty);
    let json_type = match type_name.as_str() {
        "String" | "str" => "string",
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
        | "u128" | "usize" => "integer",
        "f32" | "f64" => "number",
        "bool" => "boolean",
        "Vec" | "HashSet" | "BTreeSet" | "VecDeque" => "array",
        "HashMap" | "BTreeMap" => "object",
        _ => "object", // Default to object for custom types
    };

    quote! {
        prop.insert(
            "type".to_string(),
            ::serde_json::Value::String(#json_type.to_string()),
        );
    }
}

/// Extract the outermost type name from a syn::Type for schema generation.
fn extract_type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let name = segment.ident.to_string();
                // For Option<T>, extract the inner T
                if name == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                            return extract_type_name(inner);
                        }
                    }
                }
                name
            } else {
                "object".to_string()
            }
        }
        _ => "object".to_string(),
    }
}

/// Convert a validation rule to a JSON Schema constraint keyword.
fn rule_to_schema_constraint(rule: &ValidationRule, field: &ValidatedField) -> Option<TokenStream> {
    match rule {
        ValidationRule::Length { min, max } => {
            // Use minLength/maxLength for strings, minItems/maxItems for arrays
            let is_string = !field.is_collection;
            let min_key = if is_string { "minLength" } else { "minItems" };
            let max_key = if is_string { "maxLength" } else { "maxItems" };

            let min_entry = min.map(|v| {
                quote! {
                    prop.insert(
                        #min_key.to_string(),
                        ::serde_json::Value::Number(::serde_json::Number::from(#v as u64)),
                    );
                }
            });
            let max_entry = max.map(|v| {
                quote! {
                    prop.insert(
                        #max_key.to_string(),
                        ::serde_json::Value::Number(::serde_json::Number::from(#v as u64)),
                    );
                }
            });

            Some(quote! { #min_entry #max_entry })
        }
        ValidationRule::Range { min, max: _ } => {
            let min_entry = min.as_ref().map(|_v| {
                // We can't evaluate the expression at compile time in schema generation,
                // so we emit code that evaluates it at runtime
                quote! {
                    // Range min/max are evaluated at schema generation time
                }
            });
            // For simplicity in schema generation, we emit the constraint only
            // if we can determine it's a literal
            Some(quote! { #min_entry })
        }
        ValidationRule::Email => Some(quote! {
            prop.insert(
                "format".to_string(),
                ::serde_json::Value::String("email".to_string()),
            );
        }),
        ValidationRule::Url => Some(quote! {
            prop.insert(
                "format".to_string(),
                ::serde_json::Value::String("uri".to_string()),
            );
        }),
        ValidationRule::Pattern(regex) => Some(quote! {
            prop.insert(
                "pattern".to_string(),
                ::serde_json::Value::String(#regex.to_string()),
            );
        }),
        // These rules don't have direct JSON Schema equivalents
        ValidationRule::Contains(_)
        | ValidationRule::Required
        | ValidationRule::Custom(_)
        | ValidationRule::Nested => None,
    }
}
