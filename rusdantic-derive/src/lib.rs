//! # rusdantic-derive
//!
//! Procedural derive macros for the Rusdantic data validation framework.
//!
//! This crate provides `#[derive(Rusdantic)]` which generates:
//! - `serde::Serialize` implementation
//! - `serde::Deserialize` implementation with embedded validation
//! - `rusdantic_core::Validate` implementation
//!
//! Users should not depend on this crate directly. Instead, use the
//! `rusdantic` facade crate which re-exports the derive macro.

extern crate proc_macro;

mod codegen;
mod diagnostics;
mod model;
mod parse;

use proc_macro::TokenStream;
use syn::DeriveInput;

/// Derive macro that generates `Serialize`, `Deserialize`, and `Validate`
/// implementations for a struct.
///
/// # Example
///
/// ```ignore
/// use rusdantic::Rusdantic;
///
/// #[derive(Rusdantic)]
/// struct User {
///     #[rusdantic(length(min = 3, max = 20))]
///     username: String,
///
///     #[rusdantic(email)]
///     email: String,
///
///     #[rusdantic(range(min = 18))]
///     age: u8,
/// }
/// ```
#[proc_macro_derive(Rusdantic, attributes(rusdantic))]
pub fn derive_rusdantic(input: TokenStream) -> TokenStream {
    // Parse the input token stream into a syn DeriveInput AST node
    let input = syn::parse_macro_input!(input as DeriveInput);

    // Delegate to the internal implementation which returns syn::Result
    // so we can accumulate and report multiple errors with proper spans
    match codegen::expand_rusdantic(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
