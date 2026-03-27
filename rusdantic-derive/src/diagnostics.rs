//! Compile-time validation of macro attribute configurations.
//!
//! This module checks for invalid configurations at compile time, providing
//! clear error messages pointing to the exact problematic attribute. For example:
//! - `range(min = 10, max = 5)` → "min must be <= max"
//! - `pattern(regex = "[invalid")` → "invalid regex pattern: ..."
//! - `length(min = -1)` → caught by Rust's type system (usize can't be negative)

use crate::parse::{LengthValidator, PatternValidator, RangeValidator, RusdanticField, RusdanticInput};

/// Validate all compile-time-checkable configurations in the parsed input.
/// Returns a vector of errors; empty means all checks passed.
pub fn validate_config(input: &RusdanticInput) -> Vec<syn::Error> {
    let mut errors = Vec::new();

    // Validate struct-level attributes
    if let Some(ref rename_all) = input.rename_all {
        if crate::model::RenameAll::from_str(rename_all).is_none() {
            errors.push(syn::Error::new(
                input.ident.span(),
                format!(
                    "unknown rename_all strategy: \"{}\". \
                     Expected one of: lowercase, UPPERCASE, camelCase, PascalCase, \
                     snake_case, SCREAMING_SNAKE_CASE, kebab-case, SCREAMING-KEBAB-CASE",
                    rename_all
                ),
            ));
        }
    }

    if let Some(ref coerce_mode) = input.coerce_mode {
        if coerce_mode != "strict" && coerce_mode != "lax" {
            errors.push(syn::Error::new(
                input.ident.span(),
                format!(
                    "unknown coerce_mode: \"{}\". Expected \"strict\" or \"lax\"",
                    coerce_mode
                ),
            ));
        }
    }

    // Validate each field's attributes
    let fields = match &input.data {
        darling::ast::Data::Struct(fields) => fields,
        _ => return errors,
    };

    for field in fields.iter() {
        validate_field(field, &mut errors);
    }

    errors
}

/// Validate a single field's attribute configuration.
fn validate_field(field: &RusdanticField, errors: &mut Vec<syn::Error>) {
    let span = field
        .ident
        .as_ref()
        .map(|i| i.span())
        .unwrap_or_else(proc_macro2::Span::call_site);

    // Validate length: min <= max when both are specified
    if let Some(ref length) = field.length {
        validate_length(length, span, errors);
    }

    // Validate range: min <= max when both are literal values
    if let Some(ref range) = field.range {
        validate_range(range, span, errors);
    }

    // Validate regex pattern syntax at compile time
    if let Some(ref pattern) = field.pattern {
        validate_pattern(pattern, span, errors);
    }

    // Warn if email/url validators are applied to non-String-like types
    // (This is a best-effort check; we can't fully resolve types at macro time)

    // Check that `required` is only used on Option<T> fields
    if field.required {
        let is_option = if let syn::Type::Path(ref tp) = field.ty {
            tp.path
                .segments
                .last()
                .map(|s| s.ident == "Option")
                .unwrap_or(false)
        } else {
            false
        };
        if !is_option {
            errors.push(syn::Error::new(
                span,
                "`required` can only be used on Option<T> fields. \
                 Non-Option fields are always required by default.",
            ));
        }
    }

    // Check for conflicting sanitizers
    if field.lowercase && field.uppercase {
        errors.push(syn::Error::new(
            span,
            "cannot use both `lowercase` and `uppercase` sanitizers on the same field",
        ));
    }
}

/// Validate that length constraints are consistent (min <= max).
fn validate_length(length: &LengthValidator, span: proc_macro2::Span, errors: &mut Vec<syn::Error>) {
    if let (Some(min), Some(max)) = (length.min, length.max) {
        if min > max {
            errors.push(syn::Error::new(
                span,
                format!(
                    "length constraint is invalid: min ({}) must be <= max ({})",
                    min, max
                ),
            ));
        }
    }
}

/// Validate that range constraints are consistent (min <= max) for literal values.
/// For non-literal expressions, we can't check at compile time and skip gracefully.
fn validate_range(
    range: &RangeValidator,
    span: proc_macro2::Span,
    errors: &mut Vec<syn::Error>,
) {
    // Try to extract literal values from min and max expressions.
    // Only integer and float literals can be compared at compile time.
    let min_val = range.min.as_ref().and_then(extract_literal_f64);
    let max_val = range.max.as_ref().and_then(extract_literal_f64);

    if let (Some(min), Some(max)) = (min_val, max_val) {
        if min > max {
            errors.push(syn::Error::new(
                span,
                format!(
                    "range constraint is invalid: min ({}) must be <= max ({})",
                    min, max
                ),
            ));
        }
    }
}

/// Try to extract a numeric literal value from a syn::Expr as f64.
/// Returns None for non-literal expressions.
fn extract_literal_f64(expr: &syn::Expr) -> Option<f64> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(i) => i.base10_parse::<f64>().ok(),
            syn::Lit::Float(f) => f.base10_parse::<f64>().ok(),
            _ => None,
        },
        // Handle negative literals: -42
        syn::Expr::Unary(unary) => {
            if matches!(unary.op, syn::UnOp::Neg(_)) {
                extract_literal_f64(&unary.expr).map(|v| -v)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Validate that a regex pattern compiles successfully at compile time.
fn validate_pattern(
    pattern: &PatternValidator,
    span: proc_macro2::Span,
    errors: &mut Vec<syn::Error>,
) {
    // Use regex-syntax crate to validate without compiling a full regex.
    // This is lighter weight and doesn't require the regex crate as a
    // dependency of the proc-macro crate.
    if let Err(e) = regex_syntax::parse(&pattern.regex) {
        errors.push(syn::Error::new(
            span,
            format!("invalid regex pattern \"{}\": {}", pattern.regex, e),
        ));
    }
}
