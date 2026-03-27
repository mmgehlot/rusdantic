//! Code generation for the `Validate` trait implementation.
//!
//! Generates `impl Validate for T` that iterates all fields, runs their
//! validation rules, accumulates errors with path tracking, and handles
//! Option<T> (skip on None), collections (iterate with index paths), and
//! nested structs (recursive validation).

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use crate::model::{ValidatedField, ValidatedStruct, ValidationRule};

/// Generate the complete `impl Validate for T` block.
pub fn generate_validate_impl(validated: &ValidatedStruct) -> TokenStream {
    let name = &validated.ident;
    let (impl_generics, ty_generics, where_clause) = validated.generics.split_for_impl();

    // Generate validation code for each field
    let field_validations: Vec<TokenStream> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_none()) // Skip computed fields
        .map(|field| generate_field_validation(field))
        .collect();

    // Generate struct-level custom validation call (cross-field validation)
    let struct_validation = validated.config.custom_validator.as_ref().map(|path| {
        quote! {
            // Run struct-level cross-field validation after all field validations
            if let Err(struct_errors) = #path(self) {
                errors.merge(struct_errors);
            }
        }
    });

    quote! {
        impl #impl_generics ::rusdantic_core::Validate for #name #ty_generics #where_clause {
            fn validate(&self) -> ::std::result::Result<(), ::rusdantic_core::ValidationErrors> {
                let mut errors = ::rusdantic_core::ValidationErrors::new();

                #(#field_validations)*

                #struct_validation

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    }
}

/// Generate validation code for a single field, handling Option<T> wrapping
/// and collection iteration.
fn generate_field_validation(field: &ValidatedField) -> TokenStream {
    let field_ident = &field.ident;
    let serialized_name = &field.serialized_name;
    let span = field.span;

    // Skip fields with no validation rules
    if field.rules.is_empty() {
        return TokenStream::new();
    }

    // Build the path segment for this field
    let path_segment = quote! {
        ::rusdantic_core::PathSegment::Field(#serialized_name.to_string())
    };

    // Generate the validation calls for each rule
    let rule_checks: Vec<TokenStream> = field
        .rules
        .iter()
        .map(|rule| generate_rule_check(rule, field, span))
        .collect();

    if field.is_option {
        // For Option<T> fields: Required checks the Option itself (outside if-let),
        // all other validators only run when the value is Some.
        let has_required = field.rules.iter().any(|r| matches!(r, ValidationRule::Required));
        let required_check = if has_required {
            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_required(
                    &self.#field_ident,
                    &[#path_segment.clone()],
                    &mut errors,
                );
            }
        } else {
            TokenStream::new()
        };

        // Filter out Required from inner checks since it's handled above
        let inner_checks: Vec<TokenStream> = field
            .rules
            .iter()
            .filter(|r| !matches!(r, ValidationRule::Required))
            .map(|rule| generate_rule_check(rule, field, span))
            .collect();

        if inner_checks.is_empty() {
            quote_spanned! { span =>
                #required_check
            }
        } else {
            quote_spanned! { span =>
                #required_check
                if let Some(ref __rusdantic_value) = self.#field_ident {
                    let __rusdantic_path = vec![#path_segment];
                    #(#inner_checks)*
                }
            }
        }
    } else if field.is_collection {
        // For collection fields: validate the collection itself AND each element
        let collection_rules: Vec<TokenStream> = field
            .rules
            .iter()
            .filter(|r| !matches!(r, ValidationRule::Nested))
            .map(|rule| generate_rule_check(rule, field, span))
            .collect();

        let element_validation = if field.nested {
            quote_spanned! { span =>
                // Validate each element in the collection
                for (__rusdantic_idx, __rusdantic_elem) in self.#field_ident.iter().enumerate() {
                    let mut __rusdantic_elem_path = vec![#path_segment.clone()];
                    __rusdantic_elem_path.push(
                        ::rusdantic_core::PathSegment::Index(__rusdantic_idx)
                    );
                    if let Err(nested_errors) = ::rusdantic_core::Validate::validate(
                        __rusdantic_elem
                    ) {
                        for mut err in nested_errors.into_errors() {
                            let mut full_path = __rusdantic_elem_path.clone();
                            full_path.extend(err.path.drain(..));
                            err.path = full_path;
                            errors.add(err);
                        }
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        quote_spanned! { span =>
            {
                let __rusdantic_path = vec![#path_segment.clone()];
                let __rusdantic_value = &self.#field_ident;
                #(#collection_rules)*
            }
            #element_validation
        }
    } else {
        // Regular field: validate directly
        let nested_validation = if field.nested {
            quote_spanned! { span =>
                // Recursively validate nested struct
                if let Err(nested_errors) = ::rusdantic_core::Validate::validate(
                    &self.#field_ident
                ) {
                    for mut err in nested_errors.into_errors() {
                        let mut full_path = vec![#path_segment.clone()];
                        full_path.extend(err.path.drain(..));
                        err.path = full_path;
                        errors.add(err);
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        quote_spanned! { span =>
            {
                let __rusdantic_path = vec![#path_segment];
                let __rusdantic_value = &self.#field_ident;
                #(#rule_checks)*
            }
            #nested_validation
        }
    }
}

/// Generate the validation check for a single rule on a field.
fn generate_rule_check(
    rule: &ValidationRule,
    field: &ValidatedField,
    span: proc_macro2::Span,
) -> TokenStream {
    // For Option<T> fields inside the `if let Some(ref value)` block,
    // we use `__rusdantic_value`. For regular fields, we also use
    // `__rusdantic_value` which is bound to `&self.field_ident`.
    match rule {
        ValidationRule::Length { min, max } => {
            let min_expr = min
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let max_expr = max
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });

            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_length(
                    __rusdantic_value,
                    #min_expr,
                    #max_expr,
                    &__rusdantic_path,
                    &mut errors,
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

            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_range(
                    __rusdantic_value,
                    #min_expr,
                    #max_expr,
                    &__rusdantic_path,
                    &mut errors,
                );
            }
        }

        ValidationRule::Email => {
            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_email(
                    __rusdantic_value,
                    &__rusdantic_path,
                    &mut errors,
                );
            }
        }

        ValidationRule::Url => {
            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_url(
                    __rusdantic_value,
                    &__rusdantic_path,
                    &mut errors,
                );
            }
        }

        ValidationRule::Pattern(regex_str) => {
            // Generate a static OnceLock<Regex> for this pattern so the regex
            // is compiled only once across all validation calls.
            let regex_lit = regex_str.as_str();
            quote_spanned! { span =>
                {
                    static __RUSDANTIC_REGEX: ::std::sync::OnceLock<::rusdantic_core::re_export::Regex> =
                        ::std::sync::OnceLock::new();
                    let regex = __RUSDANTIC_REGEX.get_or_init(|| {
                        ::rusdantic_core::re_export::Regex::new(#regex_lit)
                            .expect("rusdantic: regex pattern was validated at compile time")
                    });
                    ::rusdantic_core::rules::validate_pattern(
                        __rusdantic_value,
                        regex,
                        #regex_lit,
                        &__rusdantic_path,
                        &mut errors,
                    );
                }
            }
        }

        ValidationRule::Contains(needle) => {
            let needle_lit = needle.as_str();
            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_contains(
                    __rusdantic_value,
                    #needle_lit,
                    &__rusdantic_path,
                    &mut errors,
                );
            }
        }

        ValidationRule::Required => {
            let field_ident = &field.ident;
            let serialized_name = &field.serialized_name;
            // For `required` on Option<T>, we check at the field level (not inside if-let)
            // This is handled specially — we need to check the original field, not the unwrapped value
            quote_spanned! { span =>
                ::rusdantic_core::rules::validate_required(
                    &self.#field_ident,
                    &[::rusdantic_core::PathSegment::Field(#serialized_name.to_string())],
                    &mut errors,
                );
            }
        }

        ValidationRule::Custom(path) => {
            let serialized_name = &field.serialized_name;
            quote_spanned! { span =>
                if let Err(mut custom_err) = #path(__rusdantic_value) {
                    custom_err.path = vec![
                        ::rusdantic_core::PathSegment::Field(#serialized_name.to_string())
                    ];
                    errors.add(custom_err);
                }
            }
        }

        // Nested validation is handled at the field level, not per-rule
        ValidationRule::Nested => TokenStream::new(),
    }
}

/// Generate a `validate_partial` method that only validates specified fields.
/// This is useful for PATCH endpoint validation where only some fields are updated.
pub fn generate_partial_validate(validated: &ValidatedStruct) -> TokenStream {
    let name = &validated.ident;
    let (impl_generics, ty_generics, where_clause) = validated.generics.split_for_impl();

    // Build match arms for each field name
    let field_arms: Vec<TokenStream> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_none() && !f.rules.is_empty())
        .map(|field| {
            let serialized_name = &field.serialized_name;
            let validation = generate_field_validation(field);
            quote! {
                #serialized_name => {
                    #validation
                }
            }
        })
        .collect();

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Validate only the specified fields by their serialized names.
            /// Useful for PATCH endpoints where only a subset of fields are updated.
            ///
            /// Fields not in the list are skipped. Unknown field names are silently ignored.
            pub fn validate_partial(
                &self,
                fields: &[&str],
            ) -> ::std::result::Result<(), ::rusdantic_core::ValidationErrors> {
                let mut errors = ::rusdantic_core::ValidationErrors::new();

                for field_name in fields {
                    match *field_name {
                        #(#field_arms)*
                        _ => {} // Silently ignore unknown field names
                    }
                }

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    }
}
