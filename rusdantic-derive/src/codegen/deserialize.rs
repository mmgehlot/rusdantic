//! Code generation for the `serde::Deserialize` trait implementation.
//!
//! Generates a custom `Deserialize` impl that follows serde's exact patterns
//! (field enum, visitor struct, visit_map) but embeds validation calls after
//! field collection and before struct construction. This ensures that invalid
//! structs never exist in memory when using `from_json()` or serde deserialization.
//!
//! The generated code also handles:
//! - Path tracking for nested error reporting
//! - Serde attribute compatibility (rename, default, skip, alias)
//! - deny_unknown_fields
//! - `Option<T>` as optional/nullable
//! - Sanitizer application during deserialization
//! - Type coercion in lax mode
//! - Field deprecation warnings

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::model::{Sanitizer, ValidatedField, ValidatedStruct, ValidationRule};

/// Generate the complete `impl Deserialize for T` block.
///
/// For generic structs, this threads type parameters through the impl block,
/// visitor struct, and adds appropriate `Deserialize` bounds on type params.
pub fn generate_deserialize_impl(validated: &ValidatedStruct) -> TokenStream {
    let name = &validated.ident;
    let name_str = name.to_string();

    let (_, ty_generics, _) = validated.generics.split_for_impl();
    let has_type_params = validated.generics.type_params().next().is_some();

    // Extract type parameter idents for PhantomData and visitor generics
    let type_param_names: Vec<&syn::Ident> = validated
        .generics
        .type_params()
        .map(|tp| &tp.ident)
        .collect();

    // For the where clause, add DeserializeOwned + Clone bounds for each type param.
    // We use DeserializeOwned (= for<'de> Deserialize<'de>) to avoid threading the
    // 'de lifetime through the entire impl which causes token parsing issues.
    let mut where_predicates: Vec<TokenStream> = Vec::new();
    // Carry forward any existing where predicates from the user's struct definition
    if let Some(ref wc) = validated.generics.where_clause {
        for pred in &wc.predicates {
            where_predicates.push(quote! { #pred });
        }
    }
    // Add our bounds for generic type params
    for ident in &type_param_names {
        where_predicates.push(quote! { #ident: ::serde::de::DeserializeOwned + Clone });
    }
    let where_clause_tokens = if where_predicates.is_empty() {
        TokenStream::new()
    } else {
        quote! { where #(#where_predicates),* }
    };

    // Build the impl params: just the user's generics params (no 'de needed for DeserializeOwned)
    let _user_generic_params: Vec<TokenStream> = validated
        .generics
        .params
        .iter()
        .map(|p| quote! { #p })
        .collect();

    // Collect non-computed fields (computed fields are not deserialized)
    let deser_fields: Vec<&ValidatedField> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_none())
        .collect();

    // Generate field enum variants and their string names for matching
    let field_variants: Vec<TokenStream> = deser_fields
        .iter()
        .map(|f| {
            let variant = format_ident!("__field_{}", f.ident);
            quote! { #variant }
        })
        .collect();

    // Generate match arms for the field deserializer: string name -> enum variant.
    // Each field's deserialization_key is the primary match, plus any aliases.
    let field_match_arms: Vec<TokenStream> = deser_fields
        .iter()
        .flat_map(|f| {
            let variant = format_ident!("__field_{}", f.ident);
            let primary = &f.deserialization_key;
            let mut arms = vec![quote! { #primary => Ok(__Field::#variant) }];
            // Add additional alias match arms
            for alias in &f.deserialization_aliases {
                arms.push(quote! { #alias => Ok(__Field::#variant) });
            }
            arms
        })
        .collect();

    // Generate the list of known field names for the `expecting` message
    let known_fields: Vec<&str> = deser_fields
        .iter()
        .map(|f| f.deserialization_key.as_str())
        .collect();
    let known_fields_array = quote! { &[#(#known_fields),*] };

    // Handle deny_unknown_fields: generate error for unknown keys
    let unknown_field_handling = if validated.config.deny_unknown_fields {
        quote! {
            _ => Err(::serde::de::Error::unknown_field(__value, #known_fields_array))
        }
    } else {
        quote! {
            _ => Ok(__Field::__ignore)
        }
    };

    // Add an __ignore variant if we don't deny unknown fields
    let ignore_variant = if !validated.config.deny_unknown_fields {
        quote! { , __ignore }
    } else {
        TokenStream::new()
    };

    // Generate field Option variables: `let mut field_name: Option<Type> = None;`
    let field_declarations: Vec<TokenStream> = deser_fields
        .iter()
        .map(|f| {
            let var = format_ident!("__field_val_{}", f.ident);
            let ty = &f.ty;
            quote! { let mut #var: Option<#ty> = None; }
        })
        .collect();

    // Generate match arms in visit_map for setting field values.
    // When coercion is enabled for a field, we use a coercing deserializer
    // instead of the default next_value().
    let field_set_arms: Vec<TokenStream> = deser_fields
        .iter()
        .map(|f| {
            let variant = format_ident!("__field_{}", f.ident);
            let var = format_ident!("__field_val_{}", f.ident);
            let serialized = &f.serialized_name;

            // Generate deprecation warning if the field is deprecated
            let deprecation_warning = f.deprecated.as_ref().map(|msg| {
                quote! {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "rusdantic warning: field '{}' is deprecated: {}",
                        #serialized, #msg
                    );
                }
            });

            // Determine if this field should use type coercion.
            // If so, generate a coercing next_value call based on the field type.
            let next_value = if f.coerce {
                generate_coercing_next_value(f)
            } else {
                quote! { __map.next_value()? }
            };

            // Handle duplicate field detection
            quote! {
                __Field::#variant => {
                    if #var.is_some() {
                        return Err(::serde::de::Error::duplicate_field(#serialized));
                    }
                    #deprecation_warning
                    #var = Some(#next_value);
                }
            }
        })
        .collect();

    // Handle __ignore variant in visit_map
    let ignore_arm = if !validated.config.deny_unknown_fields {
        quote! {
            __Field::__ignore => {
                let _ = __map.next_value::<::serde::de::IgnoredAny>()?;
            }
        }
    } else {
        TokenStream::new()
    };

    // Generate field extraction: unwrap Option or use default
    let field_extractions: Vec<TokenStream> = deser_fields
        .iter()
        .map(|f| {
            let var = format_ident!("__field_val_{}", f.ident);
            let field_ident = &f.ident;
            let serialized = &f.serialized_name;

            if f.is_option {
                // Option<T> fields: None is a valid default
                quote! {
                    let #field_ident = #var.unwrap_or(None);
                }
            } else {
                // Required fields: error if missing
                quote! {
                    let #field_ident = #var.ok_or_else(|| {
                        ::serde::de::Error::missing_field(#serialized)
                    })?;
                }
            }
        })
        .collect();

    // Generate sanitizer applications
    let sanitizer_applications: Vec<TokenStream> = deser_fields
        .iter()
        .filter(|f| !f.sanitizers.is_empty())
        .map(|f| generate_sanitizers(f))
        .collect();

    // Generate validation calls (same logic as Validate impl)
    let validation_calls: Vec<TokenStream> = deser_fields
        .iter()
        .filter(|f| !f.rules.is_empty())
        .map(|f| generate_deser_field_validation(f))
        .collect();

    // Struct-level custom validation runs AFTER struct construction (no cloning).
    // We construct the struct, then validate, then return or error.
    let struct_post_validation = validated.config.custom_validator.as_ref().map(|path| {
        quote! {
            if let Err(struct_errors) = #path(&__result) {
                return Err(::serde::de::Error::custom(struct_errors));
            }
        }
    });

    // Generate the struct construction from validated fields
    let struct_construction: Vec<TokenStream> = deser_fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote! { #ident }
        })
        .collect();

    // For generic structs, the visitor needs PhantomData to use the type params.
    // For non-generic structs, we don't need PhantomData at all.
    let visitor_phantom = if has_type_params {
        quote! { __phantom: ::std::marker::PhantomData<(#(#type_param_names),*)> }
    } else {
        TokenStream::new()
    };
    let visitor_phantom_init = if has_type_params {
        quote! { __phantom: ::std::marker::PhantomData }
    } else {
        TokenStream::new()
    };
    let visitor_struct_generics = if has_type_params {
        quote! { <#(#type_param_names),*> }
    } else {
        TokenStream::new()
    };

    // LIMITATION: Generic structs do not get a custom Deserialize impl.
    // Users must add #[derive(serde::Deserialize)] manually and call .validate()
    // after deserialization. This is a known limitation documented in README.md.
    if has_type_params {
        return TokenStream::new();
    }

    // Non-generic structs: generate the full custom Deserialize impl with embedded validation.
    let deser_impl_start = quote! { impl<'de> };

    quote! {
        #deser_impl_start ::serde::Deserialize<'de> for #name #ty_generics
            #where_clause_tokens
        {
            fn deserialize<__D>(__deserializer: __D) -> ::std::result::Result<Self, __D::Error>
            where
                __D: ::serde::Deserializer<'de>,
            {
                // Import IntoDeserializer for type coercion support
                use ::serde::de::IntoDeserializer as _;

                // Step 1: Define field identifier enum for key matching
                #[allow(non_camel_case_types)]
                enum __Field {
                    #(#field_variants),*
                    #ignore_variant
                }

                // Step 2: Implement Deserialize for the field enum
                impl<'de> ::serde::Deserialize<'de> for __Field {
                    fn deserialize<__D>(__deserializer: __D) -> ::std::result::Result<Self, __D::Error>
                    where
                        __D: ::serde::Deserializer<'de>,
                    {
                        struct __FieldVisitor;

                        impl<'de> ::serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;

                            fn expecting(
                                &self,
                                __formatter: &mut ::std::fmt::Formatter,
                            ) -> ::std::fmt::Result {
                                ::std::fmt::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }

                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> ::std::result::Result<__Field, __E>
                            where
                                __E: ::serde::de::Error,
                            {
                                match __value {
                                    #(#field_match_arms,)*
                                    #unknown_field_handling
                                }
                            }
                        }

                        __deserializer.deserialize_identifier(__FieldVisitor)
                    }
                }

                // Step 3: Define the visitor struct (generic over type params if any)
                struct __Visitor #visitor_struct_generics {
                    #visitor_phantom
                }

                #deser_impl_start ::serde::de::Visitor<'de> for __Visitor #visitor_struct_generics
                    #where_clause_tokens
                {
                    type Value = #name #ty_generics;

                    fn expecting(
                        &self,
                        __formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        ::std::fmt::Formatter::write_str(
                            __formatter,
                            &format!("struct {}", #name_str),
                        )
                    }

                    fn visit_map<__M>(
                        self,
                        mut __map: __M,
                    ) -> ::std::result::Result<#name #ty_generics, __M::Error>
                    where
                        __M: ::serde::de::MapAccess<'de>,
                    {
                        // Declare Option variables for each field
                        #(#field_declarations)*

                        // Iterate over map keys and collect field values
                        while let Some(__key) = __map.next_key::<__Field>()? {
                            match __key {
                                #(#field_set_arms)*
                                #ignore_arm
                            }
                        }

                        // Extract field values, erroring on missing required fields
                        #(#field_extractions)*

                        // Apply sanitizers to string fields
                        #(#sanitizer_applications)*

                        // Run field-level validation, collecting all errors
                        let mut __errors = ::rusdantic_core::ValidationErrors::new();
                        #(#validation_calls)*

                        // Return field validation errors if any were found
                        if !__errors.is_empty() {
                            return Err(::serde::de::Error::custom(__errors));
                        }

                        // Construct the struct (all field validations passed)
                        let __result = #name #ty_generics {
                            #(#struct_construction,)*
                        };

                        // Run struct-level cross-field validation on the
                        // constructed struct (no cloning needed)
                        #struct_post_validation

                        Ok(__result)
                    }
                }

                // Step 4: Invoke deserialization with our visitor
                __deserializer.deserialize_struct(
                    #name_str,
                    #known_fields_array,
                    __Visitor { #visitor_phantom_init },
                )
            }
        }
    }
}

/// Generate sanitizer application code for a field during deserialization.
/// Sanitizers mutate the field value after deserialization but before validation.
fn generate_sanitizers(field: &ValidatedField) -> TokenStream {
    let field_ident = &field.ident;

    let sanitizer_ops: Vec<TokenStream> = field
        .sanitizers
        .iter()
        .map(|s| match s {
            Sanitizer::Trim => {
                if field.is_option {
                    quote! {
                        if let Some(ref mut v) = #field_ident {
                            *v = v.trim().to_string();
                        }
                    }
                } else {
                    quote! {
                        let #field_ident = #field_ident.trim().to_string();
                    }
                }
            }
            Sanitizer::Lowercase => {
                if field.is_option {
                    quote! {
                        if let Some(ref mut v) = #field_ident {
                            *v = v.to_lowercase();
                        }
                    }
                } else {
                    quote! {
                        let #field_ident = #field_ident.to_lowercase();
                    }
                }
            }
            Sanitizer::Uppercase => {
                if field.is_option {
                    quote! {
                        if let Some(ref mut v) = #field_ident {
                            *v = v.to_uppercase();
                        }
                    }
                } else {
                    quote! {
                        let #field_ident = #field_ident.to_uppercase();
                    }
                }
            }
            Sanitizer::Truncate(max) => {
                if field.is_option {
                    quote! {
                        if let Some(ref mut v) = #field_ident {
                            if v.chars().count() > #max {
                                *v = v.chars().take(#max).collect();
                            }
                        }
                    }
                } else {
                    quote! {
                        let #field_ident = if #field_ident.chars().count() > #max {
                            #field_ident.chars().take(#max).collect()
                        } else {
                            #field_ident
                        };
                    }
                }
            }
            Sanitizer::Custom(path) => {
                if field.is_option {
                    quote! {
                        if let Some(ref mut v) = #field_ident {
                            *v = #path(v.clone());
                        }
                    }
                } else {
                    quote! {
                        let #field_ident = #path(#field_ident);
                    }
                }
            }
        })
        .collect();

    quote! { #(#sanitizer_ops)* }
}

/// Generate validation code for a field during deserialization.
/// Similar to validate.rs but operates on local variables instead of self.field.
fn generate_deser_field_validation(field: &ValidatedField) -> TokenStream {
    let field_ident = &field.ident;
    let serialized_name = &field.serialized_name;

    let path_segment = quote! {
        ::rusdantic_core::PathSegment::Field(#serialized_name.to_string())
    };

    let rule_checks: Vec<TokenStream> = field
        .rules
        .iter()
        .filter(|r| !matches!(r, ValidationRule::Required | ValidationRule::Nested))
        .map(|rule| generate_deser_rule_check(rule, field))
        .collect();

    // Handle `required` separately since it checks the Option itself
    let required_check = if field
        .rules
        .iter()
        .any(|r| matches!(r, ValidationRule::Required))
    {
        quote! {
            ::rusdantic_core::rules::validate_required(
                &#field_ident,
                &[#path_segment.clone()],
                &mut __errors,
            );
        }
    } else {
        TokenStream::new()
    };

    // Handle nested validation
    let nested_check = if field.nested && !field.is_collection {
        if field.is_option {
            quote! {
                if let Some(ref __nested) = #field_ident {
                    if let Err(nested_errors) = ::rusdantic_core::Validate::validate(__nested) {
                        for mut err in nested_errors.into_errors() {
                            let mut full_path = vec![#path_segment.clone()];
                            full_path.append(&mut err.path);
                            err.path = full_path;
                            __errors.add(err);
                        }
                    }
                }
            }
        } else {
            quote! {
                if let Err(nested_errors) = ::rusdantic_core::Validate::validate(&#field_ident) {
                    for mut err in nested_errors.into_errors() {
                        let mut full_path = vec![#path_segment.clone()];
                        full_path.append(&mut err.path);
                        err.path = full_path;
                        __errors.add(err);
                    }
                }
            }
        }
    } else if field.nested && field.is_collection {
        quote! {
            for (__idx, __elem) in #field_ident.iter().enumerate() {
                if let Err(nested_errors) = ::rusdantic_core::Validate::validate(__elem) {
                    for mut err in nested_errors.into_errors() {
                        let mut full_path = vec![
                            #path_segment.clone(),
                            ::rusdantic_core::PathSegment::Index(__idx),
                        ];
                        full_path.append(&mut err.path);
                        err.path = full_path;
                        __errors.add(err);
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };

    if field.is_option && !rule_checks.is_empty() {
        quote! {
            #required_check
            if let Some(ref __rusdantic_value) = #field_ident {
                let __rusdantic_path = vec![#path_segment];
                #(#rule_checks)*
            }
            #nested_check
        }
    } else if !rule_checks.is_empty() {
        quote! {
            {
                let __rusdantic_value = &#field_ident;
                let __rusdantic_path = vec![#path_segment];
                #(#rule_checks)*
            }
            #nested_check
        }
    } else {
        quote! {
            #required_check
            #nested_check
        }
    }
}

/// Generate a single validation rule check for deserialization context.
fn generate_deser_rule_check(rule: &ValidationRule, field: &ValidatedField) -> TokenStream {
    match rule {
        ValidationRule::Length { min, max } => {
            let min_expr = min
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let max_expr = max
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });

            quote! {
                ::rusdantic_core::rules::validate_length(
                    __rusdantic_value,
                    #min_expr,
                    #max_expr,
                    &__rusdantic_path,
                    &mut __errors,
                );
            }
        }
        ValidationRule::Range { min, max } => {
            let min_expr = min
                .as_ref()
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let max_expr = max
                .as_ref()
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });

            quote! {
                ::rusdantic_core::rules::validate_range(
                    __rusdantic_value,
                    #min_expr,
                    #max_expr,
                    &__rusdantic_path,
                    &mut __errors,
                );
            }
        }
        ValidationRule::Email => {
            quote! {
                ::rusdantic_core::rules::validate_email(
                    __rusdantic_value,
                    &__rusdantic_path,
                    &mut __errors,
                );
            }
        }
        ValidationRule::Url => {
            quote! {
                ::rusdantic_core::rules::validate_url(
                    __rusdantic_value,
                    &__rusdantic_path,
                    &mut __errors,
                );
            }
        }
        ValidationRule::Pattern(regex_str) => {
            let regex_lit = regex_str.as_str();
            quote! {
                {
                    static __RUSDANTIC_REGEX: ::std::sync::OnceLock<::rusdantic_core::re_export::Regex> =
                        ::std::sync::OnceLock::new();
                    let regex = __RUSDANTIC_REGEX.get_or_init(|| {
                        ::rusdantic_core::re_export::Regex::new(
                            &::rusdantic_core::rules::pattern::anchor_pattern(#regex_lit)
                        )
                            .expect("rusdantic: regex validated at compile time")
                    });
                    ::rusdantic_core::rules::validate_pattern(
                        __rusdantic_value,
                        regex,
                        #regex_lit,
                        &__rusdantic_path,
                        &mut __errors,
                    );
                }
            }
        }
        ValidationRule::Contains(needle) => {
            let needle_lit = needle.as_str();
            quote! {
                ::rusdantic_core::rules::validate_contains(
                    __rusdantic_value,
                    #needle_lit,
                    &__rusdantic_path,
                    &mut __errors,
                );
            }
        }
        ValidationRule::Custom(path, _mode) => {
            let serialized_name = &field.serialized_name;
            quote! {
                if let Err(mut custom_err) = #path(__rusdantic_value) {
                    custom_err.path = vec![
                        ::rusdantic_core::PathSegment::Field(#serialized_name.to_string())
                    ];
                    __errors.add(custom_err);
                }
            }
        }
        ValidationRule::CustomWithContext(_) => {
            // Context-aware validators only run via validate_with_context()
            TokenStream::new()
        }
        // Required and Nested are handled separately
        ValidationRule::Required | ValidationRule::Nested => TokenStream::new(),
    }
}

/// Generate a coercing `next_value` call for a field based on its type.
///
/// For `Option<T>` fields: reads a `serde_json::Value` first, checks for null
/// (returning `None`), then coerces the non-null value and wraps in `Some`.
/// This returns `Option<T>` so the caller's `Some(result)` produces `Option<`Option<T>`>`.
///
/// For non-Option fields: reads and coerces directly, returning `T`.
fn generate_coercing_next_value(field: &ValidatedField) -> TokenStream {
    let ty = &field.ty;
    let inner_type_name = extract_inner_type_name(ty);
    let inner_ty = extract_inner_rust_type(ty);

    // Generate the coercion expression that converts a serde_json::Value → T
    let coerce_expr = match inner_type_name.as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => {
            quote! {
                {
                    let __coerced: #inner_ty = ::rusdantic_core::coerce::deserialize_coerce_int(
                        ::serde::de::IntoDeserializer::into_deserializer(__raw)
                    ).map_err(::serde::de::Error::custom)?;
                    __coerced
                }
            }
        }
        "f32" | "f64" => {
            quote! {
                {
                    let __coerced: #inner_ty = ::rusdantic_core::coerce::deserialize_coerce_float(
                        ::serde::de::IntoDeserializer::into_deserializer(__raw)
                    ).map_err(::serde::de::Error::custom)?;
                    __coerced
                }
            }
        }
        "bool" => {
            quote! {
                ::rusdantic_core::coerce::deserialize_coerce_bool(
                    ::serde::de::IntoDeserializer::into_deserializer(__raw)
                ).map_err(::serde::de::Error::custom)?
            }
        }
        "String" => {
            quote! {
                ::rusdantic_core::coerce::deserialize_coerce_string(
                    ::serde::de::IntoDeserializer::into_deserializer(__raw)
                ).map_err(::serde::de::Error::custom)?
            }
        }
        // Unknown types: fall back to standard deserialization
        _ => {
            return quote! { __map.next_value()? };
        }
    };

    if field.is_option {
        // For Option<T>: read raw value, handle null, coerce non-null.
        // Returns Option<T> so caller's Some(result) gives Option<Option<T>>.
        quote! {
            {
                let __raw = __map.next_value::<::serde_json::Value>()?;
                if __raw.is_null() {
                    None
                } else {
                    Some(#coerce_expr)
                }
            }
        }
    } else {
        // For non-Option<T>: read raw value and coerce. Returns T.
        quote! {
            {
                let __raw = __map.next_value::<::serde_json::Value>()?;
                #coerce_expr
            }
        }
    }
}

/// Extract the inner Rust type, unwrapping `Option<T>` to get T.
/// Used by coercion codegen to annotate the coerced value's type.
fn extract_inner_rust_type(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return inner;
                    }
                }
            }
        }
    }
    ty
}

/// Extract the type name from a syn::Type, handling `Option<T>` by returning T.
fn extract_inner_type_name(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let name = segment.ident.to_string();
            // For Option<T>, extract T
            if name == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return extract_inner_type_name(inner);
                    }
                }
            }
            return name;
        }
    }
    "unknown".to_string()
}
