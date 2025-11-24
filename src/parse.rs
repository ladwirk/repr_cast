//! Parsing logic for repr_cast macro.
//!
//! This module is responsible for:
//! - Parsing the repr type from the attribute arguments
//! - Validating the enum structure (must be fieldless)
//! - Extracting enum metadata (name, visibility, attributes, variants)
//! - Computing discriminant values for variants

use crate::repr_enum::{CalculatedDiscriminant, EnumVariant, ReprEnum};
use syn::{Data, DeriveInput, Error, Fields, Ident, Meta, Result};

/// Parse the repr_cast macro input.
///
/// # Arguments
/// * `args` - The attribute arguments (the repr type)
/// * `input` - The enum definition
///
/// # Returns
/// A validated `ReprEnum` ready for code generation, or an error if validation fails.
pub fn parse_repr_cast(args: Ident, input: DeriveInput) -> Result<ReprEnum> {
    let repr_type = args;

    // Validate that we're working with an enum
    let enum_data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "repr_cast can only be applied to enums",
            ))
        }
    };

    // Validate all variants are fieldless
    for variant in &enum_data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                variant,
                "repr_cast can only be applied to fieldless enums (enums with unit variants)",
            ));
        }
    }

    // Calculate discriminants for all variants
    let variants = calculate_discriminants(&enum_data.variants)?;

    // Filter out repr attributes that match our repr_type to avoid duplication
    let attributes = input
        .attrs
        .into_iter()
        .filter(|attr| !is_matching_repr_attr(attr, &repr_type))
        .collect();

    Ok(ReprEnum {
        name: input.ident,
        repr_type,
        visibility: input.vis,
        attributes,
        generics: input.generics,
        variants,
    })
}

/// Parse the repr type from existing #[repr(...)] attributes if no args provided.
pub fn extract_repr_from_attrs(input: &DeriveInput) -> Result<Option<Ident>> {
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = &meta_list.tokens;
                let repr_type: Ident = syn::parse2(tokens.clone())?;
                return Ok(Some(repr_type));
            }
        }
    }
    Ok(None)
}

/// Check if an attribute is a #[repr(T)] attribute matching the given type.
fn is_matching_repr_attr(attr: &syn::Attribute, repr_type: &Ident) -> bool {
    if attr.path().is_ident("repr") {
        if let Meta::List(meta_list) = &attr.meta {
            if let Ok(existing_type) = syn::parse2::<Ident>(meta_list.tokens.clone()) {
                return existing_type == *repr_type;
            }
        }
    }
    false
}

/// Calculate discriminants for all variants, handling both explicit and implicit values.
fn calculate_discriminants(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Result<Vec<EnumVariant>> {
    let mut result = Vec::new();
    let mut next_implicit_value: i128 = 0;

    for variant in variants.iter() {
        let calculated_discriminant = if let Some((_, expr)) = &variant.discriminant {
            // Explicit discriminant - try to evaluate it for the next implicit value
            if let Ok(value) = try_evaluate_expr(expr) {
                next_implicit_value = value + 1;
            } else {
                // Can't evaluate - reset implicit counter
                // This handles complex expressions like consts
                next_implicit_value = 0;
            }
            CalculatedDiscriminant::Explicit(expr.clone())
        } else {
            // Implicit discriminant
            let value = next_implicit_value;
            next_implicit_value += 1;
            CalculatedDiscriminant::Implicit(value)
        };

        result.push(EnumVariant {
            name: variant.ident.clone(),
            attributes: variant.attrs.clone(),
            discriminant: variant.discriminant.as_ref().map(|(_, expr)| expr.clone()),
            calculated_discriminant,
        });
    }

    Ok(result)
}

/// Try to evaluate a simple expression to an i128 value.
/// This is best-effort and only handles simple literal cases.
fn try_evaluate_expr(expr: &syn::Expr) -> Result<i128> {
    match expr {
        syn::Expr::Lit(lit_expr) => match &lit_expr.lit {
            syn::Lit::Int(lit_int) => lit_int
                .base10_parse::<i128>()
                .map_err(|e| Error::new_spanned(lit_int, e)),
            _ => Err(Error::new_spanned(expr, "expected integer literal")),
        },
        syn::Expr::Unary(unary) => match &unary.op {
            syn::UnOp::Neg(_) => {
                let value = try_evaluate_expr(&unary.expr)?;
                Ok(-value)
            }
            _ => Err(Error::new_spanned(expr, "only negation is supported")),
        },
        _ => Err(Error::new_spanned(
            expr,
            "cannot evaluate complex expression",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_simple_enum() {
        let input: DeriveInput = parse_quote! {
            enum Status {
                Pending = 0,
                Active = 1,
                Completed = 2,
            }
        };

        let repr_type: Ident = parse_quote! { u8 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_ok());
        let repr_enum = result.unwrap();
        assert_eq!(repr_enum.name.to_string(), "Status");
        assert_eq!(repr_enum.repr_type.to_string(), "u8");
        assert_eq!(repr_enum.variants.len(), 3);
    }

    #[test]
    fn test_parse_implicit_discriminants() {
        let input: DeriveInput = parse_quote! {
            enum Color {
                Red,
                Green,
                Blue,
            }
        };

        let repr_type: Ident = parse_quote! { u8 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_ok());
        let repr_enum = result.unwrap();
        assert_eq!(repr_enum.variants.len(), 3);

        // Check implicit discriminants are calculated correctly
        match &repr_enum.variants[0].calculated_discriminant {
            CalculatedDiscriminant::Implicit(v) => assert_eq!(*v, 0),
            _ => panic!("Expected implicit discriminant"),
        }
        match &repr_enum.variants[1].calculated_discriminant {
            CalculatedDiscriminant::Implicit(v) => assert_eq!(*v, 1),
            _ => panic!("Expected implicit discriminant"),
        }
        match &repr_enum.variants[2].calculated_discriminant {
            CalculatedDiscriminant::Implicit(v) => assert_eq!(*v, 2),
            _ => panic!("Expected implicit discriminant"),
        }
    }

    #[test]
    fn test_parse_signed_discriminants() {
        let input: DeriveInput = parse_quote! {
            enum Priority {
                Low = -1,
                Normal = 0,
                High = 1,
            }
        };

        let repr_type: Ident = parse_quote! { i32 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_ok());
        let repr_enum = result.unwrap();
        assert_eq!(repr_enum.variants.len(), 3);
        assert_eq!(repr_enum.repr_type.to_string(), "i32");
    }

    #[test]
    fn test_parse_mixed_discriminants() {
        let input: DeriveInput = parse_quote! {
            enum Mixed {
                First,
                Second = 10,
                Third,
            }
        };

        let repr_type: Ident = parse_quote! { u16 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_ok());
        let repr_enum = result.unwrap();

        match &repr_enum.variants[0].calculated_discriminant {
            CalculatedDiscriminant::Implicit(v) => assert_eq!(*v, 0),
            _ => panic!("Expected implicit discriminant"),
        }
        // Second is explicit
        assert!(matches!(
            &repr_enum.variants[1].calculated_discriminant,
            CalculatedDiscriminant::Explicit(_)
        ));
        // Third should be 11 (implicit after 10)
        match &repr_enum.variants[2].calculated_discriminant {
            CalculatedDiscriminant::Implicit(v) => assert_eq!(*v, 11),
            _ => panic!("Expected implicit discriminant"),
        }
    }

    #[test]
    fn test_parse_rejects_non_enum() {
        let input: DeriveInput = parse_quote! {
            struct NotAnEnum {
                field: u32,
            }
        };

        let repr_type: Ident = parse_quote! { u8 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("can only be applied to enums"));
    }

    #[test]
    fn test_parse_rejects_enum_with_fields() {
        let input: DeriveInput = parse_quote! {
            enum WithFields {
                Variant1(u32),
                Variant2,
            }
        };

        let repr_type: Ident = parse_quote! { u8 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("fieldless enums"));
    }

    #[test]
    fn test_extract_repr_from_attrs() {
        let input: DeriveInput = parse_quote! {
            #[repr(u32)]
            enum Test {
                A,
            }
        };

        let result = extract_repr_from_attrs(&input).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "u32");
    }

    #[test]
    fn test_extract_repr_none() {
        let input: DeriveInput = parse_quote! {
            enum Test {
                A,
            }
        };

        let result = extract_repr_from_attrs(&input).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_evaluate_expr_positive() {
        let expr: syn::Expr = parse_quote! { 42 };
        let result = try_evaluate_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_try_evaluate_expr_negative() {
        let expr: syn::Expr = parse_quote! { -10 };
        let result = try_evaluate_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -10);
    }

    #[test]
    fn test_try_evaluate_expr_complex() {
        let expr: syn::Expr = parse_quote! { 1 + 2 };
        let result = try_evaluate_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_preserves_attributes() {
        let input: DeriveInput = parse_quote! {
            #[derive(Debug)]
            #[allow(dead_code)]
            enum Test {
                A,
            }
        };

        let repr_type: Ident = parse_quote! { u8 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_ok());
        let repr_enum = result.unwrap();
        assert_eq!(repr_enum.attributes.len(), 2);
    }

    #[test]
    fn test_filters_matching_repr_attr() {
        let input: DeriveInput = parse_quote! {
            #[repr(u8)]
            #[derive(Debug)]
            enum Test {
                A,
            }
        };

        let repr_type: Ident = parse_quote! { u8 };
        let result = parse_repr_cast(repr_type, input);

        assert!(result.is_ok());
        let repr_enum = result.unwrap();
        // Should only have Debug, not the repr(u8) since we'll add our own
        assert_eq!(repr_enum.attributes.len(), 1);
    }
}
