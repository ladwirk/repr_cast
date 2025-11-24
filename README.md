# repr_cast

A Rust procedural macro library that enhances fieldless enums with proper conversions between enum variants and their integer representation types.

## Features

- **Bidirectional conversions**: Convert enums to integers and integers back to enums
- **Type-safe**: Uses `TryFrom` for fallible conversions from integers
- **Const-friendly**: Generated methods work in const contexts
- **Ergonomic**: Supports both explicit and implicit discriminants
- **Error handling**: Provides descriptive error types for invalid conversions
- **Zero overhead**: All conversions are inlined and compile to efficient code

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
repr_cast = "0.1"
```

## Usage

Simply add the `#[repr_cast(T)]` attribute to your fieldless enum, where `T` is the integer type you want to use:

```rust
use repr_cast::repr_cast;

#[derive(Debug, PartialEq)]
#[repr_cast(u8)]
enum Status {
    Pending = 0,
    Active = 1,
    Completed = 2,
}
```

### Generated API

The macro generates the following for your enum:

1. **`#[repr(T)]`** - Ensures the enum has the specified memory representation
2. **`From<Enum> for T`** - Converts enum to integer
3. **`TryFrom<T> for Enum`** - Converts integer to enum (returns `EnumConversionError` for invalid values)
4. **`Enum::from_repr(value: T) -> Option<Enum>`** - Safe conversion from integer
5. **`Enum::as_repr(&self) -> T`** - Converts enum to integer
6. **`EnumConversionError`** - Error type for failed conversions

### Examples

#### Basic conversion

```rust
use repr_cast::repr_cast;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
#[repr_cast(u8)]
enum Status {
    Pending = 0,
    Active = 1,
    Completed = 2,
}

// Enum to integer
let status = Status::Active;
let value: u8 = status.into();  // or status.as_repr()
assert_eq!(value, 1);

// Integer to enum (fallible)
let status = Status::try_from(1).unwrap();
assert_eq!(status, Status::Active);

// Safe conversion
if let Some(status) = Status::from_repr(2) {
    println!("Got status: {:?}", status);
}
```

#### Signed integers

```rust
#[derive(Debug, PartialEq)]
#[repr_cast(i32)]
enum Priority {
    Low = -1,
    Normal = 0,
    High = 1,
    Critical = 100,
}

let priority = Priority::Low;
let value: i32 = priority.into();
assert_eq!(value, -1);
```

#### Implicit discriminants

```rust
#[derive(Debug, PartialEq)]
#[repr_cast(u16)]
enum Color {
    Red,    // 0
    Green,  // 1
    Blue,   // 2
}

assert_eq!(Color::Green.as_repr(), 1);
```

#### Const expressions

```rust
const ERROR_BASE: u16 = 400;
const SERVER_ERROR_BASE: u16 = 500;

#[derive(Debug, PartialEq)]
#[repr_cast(u16)]
enum ErrorCode {
    Success = 0,
    InvalidInput,                  // 1
    BadRequest = ERROR_BASE,       // 400
    Forbidden,                     // 401
    InternalError = SERVER_ERROR_BASE, // 500
    ServiceUnavailable,            // 501
}

assert_eq!(ErrorCode::BadRequest.as_repr(), ERROR_BASE);
assert_eq!(ErrorCode::try_from(500).unwrap(), ErrorCode::InternalError);
```

#### Const contexts

```rust
#[derive(Debug, PartialEq)]
#[repr_cast(u8)]
enum Flag {
    Off = 0,
    On = 1,
}

const FLAG_VALUE: u8 = Flag::On.as_repr();
const MAYBE_FLAG: Option<Flag> = Flag::from_repr(1);
```

#### Error handling

```rust
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
#[repr_cast(u8)]
enum Status {
    Pending = 0,
    Active = 1,
}

match Status::try_from(99) {
    Ok(status) => println!("Valid: {:?}", status),
    Err(err) => {
        println!("Error: {}", err);  // "unknown Status variant: 99"
        println!("Invalid value: {}", err.0);  // 99
    }
}
```

## Supported Integer Types

The macro works with all Rust integer types:

- Unsigned: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- Signed: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`

## Requirements

- The enum must be **fieldless** (all variants must be unit variants)
- The enum cannot have generics (currently)
- All discriminant values must fit in the specified integer type

## Comparison with `#[repr(i*)]` / `#[repr(u*)]`

Rust's built-in `#[repr(i*)]` and `#[repr(u*)]` attributes only control memory layout. They don't provide any conversion methods. This library:

- ✅ Includes `#[repr(T)]` automatically
- ✅ Adds safe conversion methods
- ✅ Implements standard traits (`From`, `TryFrom`)
- ✅ Provides const-compatible functions
- ✅ Includes proper error types with helpful messages

## License

This project is dual-licensed under MIT or Apache-2.0, at your option.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
