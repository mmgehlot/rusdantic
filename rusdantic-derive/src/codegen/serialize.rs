//! Code generation for the `serde::Serialize` trait implementation.
//!
//! Generates a standard serde `Serialize` impl that respects rename_all
//! and computed fields. No validation is performed during serialization —
//! validation is a deserialization-time and construction-time concern.

use proc_macro2::TokenStream;
use quote::quote;

use crate::model::ValidatedStruct;

/// Generate the complete `impl Serialize for T` block.
///
/// For generic structs, adds `T: Serialize` bounds on all type parameters.
pub fn generate_serialize_impl(validated: &ValidatedStruct) -> TokenStream {
    let name = &validated.ident;
    let (_impl_generics, ty_generics, where_clause) = validated.generics.split_for_impl();
    let name_str = name.to_string();

    // For generic structs, we need to add `Serialize` bounds to each type param.
    // We clone the generics and add the extra bound to each type param.
    let mut ser_generics = validated.generics.clone();
    for param in ser_generics.type_params_mut() {
        param.bounds.push(syn::parse_quote!(::serde::Serialize));
    }
    let (ser_impl_generics, _, _) = ser_generics.split_for_impl();

    // Count total fields including computed fields
    let regular_fields: Vec<_> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_none())
        .collect();

    let computed_fields: Vec<_> = validated
        .fields
        .iter()
        .filter(|f| f.computed_method.is_some())
        .collect();

    let total_fields = regular_fields.len() + computed_fields.len();

    // Generate serialize_field calls for regular fields
    let regular_field_serializations: Vec<TokenStream> = regular_fields
        .iter()
        .map(|field| {
            let field_ident = &field.ident;
            let serialized_name = &field.serialized_name;
            quote! {
                __state.serialize_field(#serialized_name, &self.#field_ident)?;
            }
        })
        .collect();

    // Generate serialize_field calls for computed fields.
    // Computed fields call a method on self to get their value.
    let computed_field_serializations: Vec<TokenStream> = computed_fields
        .iter()
        .map(|field| {
            let serialized_name = &field.serialized_name;
            let method_name = field
                .computed_method
                .as_ref()
                .expect("computed field must have a method name");
            let method_ident = syn::Ident::new(method_name, field.ident.span());
            quote! {
                __state.serialize_field(#serialized_name, &self.#method_ident())?;
            }
        })
        .collect();

    quote! {
        impl #ser_impl_generics ::serde::Serialize for #name #ty_generics #where_clause {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> ::std::result::Result<__S::Ok, __S::Error>
            where
                __S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeStruct;
                let mut __state = __serializer.serialize_struct(#name_str, #total_fields)?;

                // Serialize regular (non-computed) fields
                #(#regular_field_serializations)*

                // Serialize computed fields by calling their methods
                #(#computed_field_serializations)*

                __state.end()
            }
        }
    }
}
