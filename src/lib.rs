use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Fields, Ident, Meta,
    Result,
};

/// An attribute macro for fieldless enums that generates conversions between
/// the enum and its integer representation type.
///
/// # Example
///
/// ```ignore
/// #[repr_cast(u8)]
/// enum Status {
///     Pending = 0,
///     Active = 1,
///     Completed = 2,
/// }
/// ```
///
/// This generates:
/// - `From<Status> for u8` - convert enum to integer
/// - `TryFrom<u8> for Status` - convert integer to enum (returns `StatusConversionError` for invalid values)
/// - `Status::from_repr(value: u8) -> Option<Status>` - safe conversion from integer
/// - `Status::as_repr(&self) -> u8` - convert enum to integer
/// - `StatusConversionError` - error type for failed conversions
#[proc_macro_attribute]
pub fn repr_cast(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the repr type from the attribute arguments
    let repr_type: Ident = if args.is_empty() {
        // If no args provided, try to extract from existing #[repr(...)] attribute
        match extract_repr_from_attrs(&input) {
            Ok(Some(ty)) => ty,
            Ok(None) => {
                return Error::new_spanned(
                    &input,
                    "repr_cast requires either an argument like #[repr_cast(i32)] or an existing #[repr(i32)] attribute"
                ).to_compile_error().into();
            }
            Err(e) => return e.to_compile_error().into(),
        }
    } else {
        parse_macro_input!(args as Ident)
    };

    match impl_repr_cast(&input, &repr_type) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn extract_repr_from_attrs(input: &DeriveInput) -> Result<Option<Ident>> {
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

fn impl_repr_cast(input: &DeriveInput, repr_type: &Ident) -> Result<TokenStream> {
    let enum_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Create a unique error type name for this enum
    let error_type_name = format_ident!("{}ConversionError", enum_name);

    // Ensure we're working with an enum
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(Error::new_spanned(
                input,
                "repr_cast can only be applied to enums",
            ));
        }
    };

    // Ensure all variants are fieldless
    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                variant,
                "repr_cast can only be applied to fieldless enums (enums with unit variants)",
            ));
        }
    }

    // Calculate discriminants for all variants
    let mut discriminants = Vec::new();
    let mut next_discriminant: i128 = 0;

    for variant in variants.iter() {
        if let Some((_, expr)) = &variant.discriminant {
            // For explicit discriminants, we'll use them directly
            discriminants.push(quote! { #expr });
            // We can't easily evaluate the expression, so we'll reset tracking
            next_discriminant = 0; // This won't be used for explicit discriminants
        } else {
            // For implicit discriminants, use the calculated value
            let lit = proc_macro2::Literal::i128_unsuffixed(next_discriminant);
            discriminants.push(quote! { #lit });
            next_discriminant += 1;
        }
    }

    // Generate match arms for converting from repr type to enum
    let from_repr_arms = variants.iter().zip(discriminants.iter()).map(|(variant, discriminant)| {
        let variant_name = &variant.ident;
        quote! {
            #discriminant => ::core::option::Option::Some(#enum_name::#variant_name),
        }
    });

    // Generate match arms for converting from enum to repr type
    // For this, we can use the enum variants directly
    let to_repr_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #enum_name::#variant_name => #enum_name::#variant_name as #repr_type,
        }
    });

    // Check if repr attribute already exists
    let repr_attr = if has_repr_attr(input, repr_type) {
        quote! {}
    } else {
        quote! { #[repr(#repr_type)] }
    };

    // Get visibility and attributes
    let vis = &input.vis;
    let attrs = &input.attrs;

    // Generate the enum definition with all variants
    let enum_variants = variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_attrs = &v.attrs;
        if let Some((_, expr)) = &v.discriminant {
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

    let expanded = quote! {
        #(#attrs)*
        #repr_attr
        #vis enum #enum_name {
            #(#enum_variants),*
        }

        impl #impl_generics #enum_name #ty_generics #where_clause {
            /// Converts an integer value to the enum variant.
            /// Returns `None` if the value doesn't match any variant.
            #[inline]
            pub const fn from_repr(value: #repr_type) -> ::core::option::Option<Self> {
                match value {
                    #(#from_repr_arms)*
                    _ => ::core::option::Option::None,
                }
            }

            /// Converts the enum variant to its integer representation.
            #[inline]
            pub const fn as_repr(&self) -> #repr_type {
                match self {
                    #(#to_repr_arms)*
                }
            }
        }

        impl #impl_generics ::core::convert::From<#enum_name #ty_generics> for #repr_type #where_clause {
            #[inline]
            fn from(value: #enum_name #ty_generics) -> Self {
                value.as_repr()
            }
        }

        impl #impl_generics ::core::convert::TryFrom<#repr_type> for #enum_name #ty_generics #where_clause {
            type Error = #error_type_name;

            #[inline]
            fn try_from(value: #repr_type) -> ::core::result::Result<Self, Self::Error> {
                Self::from_repr(value).ok_or(#error_type_name(value))
            }
        }

        /// Error type returned when trying to convert an integer to this enum
        /// but the value doesn't match any known variant.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis struct #error_type_name(pub #repr_type);

        impl ::core::fmt::Display for #error_type_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "unknown {} variant: {}", stringify!(#enum_name), self.0)
            }
        }

        impl ::core::error::Error for #error_type_name {}
    };

    Ok(expanded.into())
}

fn has_repr_attr(input: &DeriveInput, repr_type: &Ident) -> bool {
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            if let Meta::List(meta_list) = &attr.meta {
                if let Ok(existing_type) = syn::parse2::<Ident>(meta_list.tokens.clone()) {
                    if existing_type == *repr_type {
                        return true;
                    }
                }
            }
        }
    }
    false
}
