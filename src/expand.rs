//! Code generation for repr_cast macro.
//!
//! This module is responsible for generating the output tokens including:
//! - The enum definition with #[repr(T)]
//! - `from_repr` and `as_repr` methods
//! - `From<Enum>` trait implementation
//! - `TryFrom<T>` trait implementation
//! - Error type definition

use crate::repr_enum::ReprEnum;
use quote::{format_ident, quote};

/// Generate the complete expanded code for a repr_cast enum.
///
/// This is the main codegen entry point that orchestrates all code generation.
pub fn expand_repr_cast(repr_enum: &ReprEnum) -> proc_macro2::TokenStream {
    let enum_def = generate_enum_definition(repr_enum);
    let impl_methods = generate_impl_methods(repr_enum);
    let from_impl = generate_from_impl(repr_enum);
    let try_from_impl = generate_try_from_impl(repr_enum);
    let error_type = generate_error_type(repr_enum);

    quote! {
        #enum_def
        #impl_methods
        #from_impl
        #try_from_impl
        #error_type
    }
}

/// Generate the enum definition with #[repr(T)] attribute.
fn generate_enum_definition(repr_enum: &ReprEnum) -> proc_macro2::TokenStream {
    let name = &repr_enum.name;
    let repr_type = &repr_enum.repr_type;
    let vis = &repr_enum.visibility;
    let attrs = &repr_enum.attributes;

    let variants = repr_enum.variants.iter().map(|v| {
        let variant_name = &v.name;
        let variant_attrs = &v.attributes;
        if let Some(expr) = &v.discriminant {
            quote! {
                #(#variant_attrs)*
                #variant_name = #expr
            }
        } else {
            quote! {
                #(#variant_attrs)*
                #variant_name
            }
        }
    });

    quote! {
        #(#attrs)*
        #[repr(#repr_type)]
        #vis enum #name {
            #(#variants),*
        }
    }
}

/// Generate the impl block with from_repr and as_repr methods.
fn generate_impl_methods(repr_enum: &ReprEnum) -> proc_macro2::TokenStream {
    let name = &repr_enum.name;
    let repr_type = &repr_enum.repr_type;
    let (impl_generics, ty_generics, where_clause) = repr_enum.generics.split_for_impl();

    // Generate if-else chain for from_repr to handle complex discriminant expressions
    // This works because we compare against the actual enum variant cast to the repr type,
    // which evaluates any expressions at compile time
    let from_repr_checks = repr_enum.variants.iter().map(|v| {
        let variant_name = &v.name;
        quote! {
            if value == #name::#variant_name as #repr_type {
                return ::core::option::Option::Some(#name::#variant_name);
            }
        }
    });

    let as_repr_arms = repr_enum.variants.iter().map(|v| {
        let variant_name = &v.name;
        quote! {
            #name::#variant_name => #name::#variant_name as #repr_type,
        }
    });

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Converts an integer value to the enum variant.
            /// Returns `None` if the value doesn't match any variant.
            #[inline]
            pub const fn from_repr(value: #repr_type) -> ::core::option::Option<Self> {
                #(#from_repr_checks)*
                ::core::option::Option::None
            }

            /// Converts the enum variant to its integer representation.
            #[inline]
            pub const fn as_repr(&self) -> #repr_type {
                match self {
                    #(#as_repr_arms)*
                }
            }
        }
    }
}

/// Generate the From<Enum> for T trait implementation.
fn generate_from_impl(repr_enum: &ReprEnum) -> proc_macro2::TokenStream {
    let name = &repr_enum.name;
    let repr_type = &repr_enum.repr_type;
    let (impl_generics, ty_generics, where_clause) = repr_enum.generics.split_for_impl();

    quote! {
        impl #impl_generics ::core::convert::From<#name #ty_generics> for #repr_type #where_clause {
            #[inline]
            fn from(value: #name #ty_generics) -> Self {
                value.as_repr()
            }
        }
    }
}

/// Generate the TryFrom<T> for Enum trait implementation.
fn generate_try_from_impl(repr_enum: &ReprEnum) -> proc_macro2::TokenStream {
    let name = &repr_enum.name;
    let repr_type = &repr_enum.repr_type;
    let error_type_name = format_ident!("{}ConversionError", name);
    let (impl_generics, ty_generics, where_clause) = repr_enum.generics.split_for_impl();

    quote! {
        impl #impl_generics ::core::convert::TryFrom<#repr_type> for #name #ty_generics #where_clause {
            type Error = #error_type_name;

            #[inline]
            fn try_from(value: #repr_type) -> ::core::result::Result<Self, Self::Error> {
                Self::from_repr(value).ok_or(#error_type_name(value))
            }
        }
    }
}

/// Generate the error type for failed conversions.
fn generate_error_type(repr_enum: &ReprEnum) -> proc_macro2::TokenStream {
    let name = &repr_enum.name;
    let repr_type = &repr_enum.repr_type;
    let vis = &repr_enum.visibility;
    let error_type_name = format_ident!("{}ConversionError", name);

    quote! {
        /// Error type returned when trying to convert an integer to this enum
        /// but the value doesn't match any known variant.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis struct #error_type_name(pub #repr_type);

        impl ::core::fmt::Display for #error_type_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "unknown {} variant: {}", stringify!(#name), self.0)
            }
        }

        impl ::core::error::Error for #error_type_name {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repr_enum::{CalculatedDiscriminant, EnumVariant};
    use syn::parse_quote;

    fn create_simple_repr_enum() -> ReprEnum {
        ReprEnum {
            name: parse_quote! { Status },
            repr_type: parse_quote! { u8 },
            visibility: parse_quote! { pub },
            attributes: vec![],
            generics: Default::default(),
            variants: vec![
                EnumVariant {
                    name: parse_quote! { Pending },
                    attributes: vec![],
                    discriminant: Some(parse_quote! { 0 }),
                    calculated_discriminant: CalculatedDiscriminant::Explicit(parse_quote! { 0 }),
                },
                EnumVariant {
                    name: parse_quote! { Active },
                    attributes: vec![],
                    discriminant: Some(parse_quote! { 1 }),
                    calculated_discriminant: CalculatedDiscriminant::Explicit(parse_quote! { 1 }),
                },
            ],
        }
    }

    #[test]
    fn test_generate_enum_definition() {
        let repr_enum = create_simple_repr_enum();
        let output = generate_enum_definition(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("# [repr (u8)]"));
        assert!(output_str.contains("pub enum Status"));
        assert!(output_str.contains("Pending = 0"));
        assert!(output_str.contains("Active = 1"));
    }

    #[test]
    fn test_generate_impl_methods() {
        let repr_enum = create_simple_repr_enum();
        let output = generate_impl_methods(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("impl Status"));
        assert!(output_str.contains("pub const fn from_repr"));
        assert!(output_str.contains("pub const fn as_repr"));
        // Now uses if-else chains instead of match
        assert!(output_str.contains("if value == Status :: Pending as u8"));
        assert!(output_str.contains("return :: core :: option :: Option :: Some (Status :: Pending)"));
    }

    #[test]
    fn test_generate_from_impl() {
        let repr_enum = create_simple_repr_enum();
        let output = generate_from_impl(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("impl :: core :: convert :: From < Status > for u8"));
        assert!(output_str.contains("value . as_repr ()"));
    }

    #[test]
    fn test_generate_try_from_impl() {
        let repr_enum = create_simple_repr_enum();
        let output = generate_try_from_impl(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("impl :: core :: convert :: TryFrom < u8 > for Status"));
        assert!(output_str.contains("type Error = StatusConversionError"));
        assert!(output_str.contains("Self :: from_repr (value)"));
    }

    #[test]
    fn test_generate_error_type() {
        let repr_enum = create_simple_repr_enum();
        let output = generate_error_type(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("pub struct StatusConversionError"));
        assert!(output_str.contains("impl :: core :: fmt :: Display"));
        assert!(output_str.contains("impl :: core :: error :: Error"));
    }

    #[test]
    fn test_expand_complete() {
        let repr_enum = create_simple_repr_enum();
        let output = expand_repr_cast(&repr_enum);
        let output_str = output.to_string();

        // Check all major components are present
        assert!(output_str.contains("pub enum Status"));
        assert!(output_str.contains("# [repr (u8)]"));
        assert!(output_str.contains("pub const fn from_repr"));
        assert!(output_str.contains("pub const fn as_repr"));
        assert!(output_str.contains("From < Status > for u8"));
        assert!(output_str.contains("TryFrom < u8 > for Status"));
        assert!(output_str.contains("StatusConversionError"));
    }

    #[test]
    fn test_implicit_discriminants() {
        let repr_enum = ReprEnum {
            name: parse_quote! { Color },
            repr_type: parse_quote! { u16 },
            visibility: parse_quote! {},
            attributes: vec![],
            generics: Default::default(),
            variants: vec![
                EnumVariant {
                    name: parse_quote! { Red },
                    attributes: vec![],
                    discriminant: None,
                    calculated_discriminant: CalculatedDiscriminant::Implicit(0),
                },
                EnumVariant {
                    name: parse_quote! { Green },
                    attributes: vec![],
                    discriminant: None,
                    calculated_discriminant: CalculatedDiscriminant::Implicit(1),
                },
            ],
        };

        let output = expand_repr_cast(&repr_enum);
        let output_str = output.to_string();

        // Implicit discriminants shouldn't have = N in the enum definition
        assert!(output_str.contains("Red"));
        assert!(output_str.contains("Green"));

        // Check the enum definition separately to ensure no explicit discriminants
        let enum_def = generate_enum_definition(&repr_enum);
        let enum_def_str = enum_def.to_string();
        assert!(!enum_def_str.contains("Red ="));
        assert!(!enum_def_str.contains("Green ="));

        // Should use if-else checks instead of match patterns
        assert!(output_str.contains("if value == Color :: Red as u16"));
        assert!(output_str.contains("if value == Color :: Green as u16"));
    }

    #[test]
    fn test_private_visibility() {
        let mut repr_enum = create_simple_repr_enum();
        repr_enum.visibility = parse_quote! {};

        let output = expand_repr_cast(&repr_enum);
        let output_str = output.to_string();

        // Should not have 'pub' before enum
        assert!(output_str.contains("enum Status"));
        // But should still have pub methods
        assert!(output_str.contains("pub const fn from_repr"));
    }

    #[test]
    fn test_with_attributes() {
        let mut repr_enum = create_simple_repr_enum();
        repr_enum.attributes = vec![parse_quote! { #[derive(Debug)] }];

        let output = generate_enum_definition(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("# [derive (Debug)]"));
    }

    #[test]
    fn test_signed_repr_type() {
        let mut repr_enum = create_simple_repr_enum();
        repr_enum.repr_type = parse_quote! { i32 };

        let output = expand_repr_cast(&repr_enum);
        let output_str = output.to_string();

        assert!(output_str.contains("# [repr (i32)]"));
        assert!(output_str.contains("From < Status > for i32"));
        assert!(output_str.contains("TryFrom < i32 > for Status"));
    }
}
