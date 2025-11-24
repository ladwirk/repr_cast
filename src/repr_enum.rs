//! Data structures representing a parsed enum suitable for repr_cast code generation.

use syn::{Attribute, Expr, Generics, Ident, Visibility};

/// Represents a fieldless enum that has been parsed and validated for repr_cast.
#[derive(Debug, Clone)]
pub struct ReprEnum {
    /// The name of the enum
    pub name: Ident,
    /// The integer type used for representation (e.g., u8, i32)
    pub repr_type: Ident,
    /// Visibility of the enum
    pub visibility: Visibility,
    /// Attributes applied to the enum (excluding repr and repr_cast)
    pub attributes: Vec<Attribute>,
    /// Generic parameters (currently not supported, kept for future extension)
    pub generics: Generics,
    /// The variants of the enum
    pub variants: Vec<EnumVariant>,
}

/// Represents a single variant in the enum.
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// The name of the variant
    pub name: Ident,
    /// Attributes applied to this variant
    pub attributes: Vec<Attribute>,
    /// The discriminant value, if explicitly specified
    pub discriminant: Option<Expr>,
    /// The calculated discriminant value (either explicit or implicit)
    pub calculated_discriminant: CalculatedDiscriminant,
}

/// Represents the calculated discriminant for a variant.
#[derive(Debug, Clone)]
pub enum CalculatedDiscriminant {
    /// An explicit discriminant expression from the source
    Explicit(Expr),
    /// An implicit discriminant (the calculated integer value)
    Implicit(i128),
}

impl CalculatedDiscriminant {
    /// Returns a token stream representing this discriminant value suitable for pattern matching.
    pub fn as_pattern_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            CalculatedDiscriminant::Explicit(expr) => quote::quote! { #expr },
            CalculatedDiscriminant::Implicit(value) => {
                let lit = proc_macro2::Literal::i128_unsuffixed(*value);
                quote::quote! { #lit }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_calculated_discriminant_explicit() {
        let expr: Expr = parse_quote! { 42 };
        let disc = CalculatedDiscriminant::Explicit(expr);
        let tokens = disc.as_pattern_tokens();
        assert_eq!(tokens.to_string(), "42");
    }

    #[test]
    fn test_calculated_discriminant_implicit() {
        let disc = CalculatedDiscriminant::Implicit(10);
        let tokens = disc.as_pattern_tokens();
        assert_eq!(tokens.to_string(), "10");
    }

    #[test]
    fn test_calculated_discriminant_negative() {
        let disc = CalculatedDiscriminant::Implicit(-5);
        let tokens = disc.as_pattern_tokens();
        assert_eq!(tokens.to_string(), "- 5");
    }
}
