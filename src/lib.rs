//! # repr_cast
//!
//! A procedural macro for fieldless enums that generates conversions between
//! enum variants and their integer representation types.
//!
//! ## Architecture
//!
//! This crate follows a clean pipeline architecture for testability:
//!
//! ```text
//! parse → expand → TokenStream
//! ```
//!
//! - **parse**: Validates input and extracts structured data
//! - **expand**: Generates output tokens from structured data
//!
//! Each stage is independently testable with unit tests.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error, Ident};

mod expand;
mod parse;
mod repr_enum;

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
/// - `From<Status> for u8` - convert owned enum to integer
/// - `From<&Status> for u8` - convert enum reference to integer
/// - `TryFrom<u8> for Status` - convert owned integer to enum (returns `StatusConversionError` for invalid values)
/// - `TryFrom<&u8> for Status` - convert integer reference to enum
/// - `Status::from_repr(value: u8) -> Option<Status>` - safe conversion from integer
/// - `Status::as_repr(&self) -> u8` - convert enum to integer
/// - `StatusConversionError` - error type for failed conversions
///
/// # Requirements
///
/// - The enum must be fieldless (all variants must be unit variants)
/// - All discriminant values must fit in the specified integer type
///
/// # Supported Integer Types
///
/// All Rust integer types are supported:
/// - Unsigned: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
/// - Signed: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
#[proc_macro_attribute]
pub fn repr_cast(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the repr type from the attribute arguments
    let repr_type: Ident = if args.is_empty() {
        // If no args provided, try to extract from existing #[repr(...)] attribute
        match parse::extract_repr_from_attrs(&input) {
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

    // Pipeline: parse → expand
    let repr_enum = match parse::parse_repr_cast(repr_type, input) {
        Ok(repr_enum) => repr_enum,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = expand::expand_repr_cast(&repr_enum);

    expanded.into()
}
