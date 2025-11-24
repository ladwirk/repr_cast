use repr_cast::repr_cast;
use std::convert::TryFrom;

// Constants for complex discriminant expressions
const ERROR_BASE: u16 = 400;
const SERVER_ERROR_BASE: u16 = 500;

// Example 1: HTTP status codes with u16 representation
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u16)]
enum HttpStatus {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    Unauthorized = 401,
    NotFound = 404,
    InternalServerError = 500,
}

// Example 2: Priority levels with i8 representation
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(i8)]
enum Priority {
    Lowest = -2,
    Low = -1,
    Normal = 0,
    High = 1,
    Critical = 2,
}

// Example 3: Simple enum with implicit discriminants
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u8)]
enum Color {
    Red,    // 0
    Green,  // 1
    Blue,   // 2
}

// Example 4: Enum with const expressions as discriminants
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u16)]
enum ErrorCode {
    Success = 0,
    InvalidInput,           // 1
    BadRequest = ERROR_BASE,  // 400
    Forbidden,              // 401
    NotFound,               // 402
    InternalError = SERVER_ERROR_BASE,  // 500
    ServiceUnavailable,     // 501
}

fn main() {
    println!("=== Example 1: HTTP Status Codes ===\n");

    // Convert enum to integer using From trait
    let status = HttpStatus::Ok;
    let status_code: u16 = status.into();
    println!("Status: {:?} -> Code: {}", status, status_code);

    // Convert enum to integer using as_repr method
    let not_found_code = HttpStatus::NotFound.as_repr();
    println!("NotFound code: {}", not_found_code);

    // Convert integer to enum using TryFrom trait
    match HttpStatus::try_from(404) {
        Ok(status) => println!("Code 404 -> Status: {:?}", status),
        Err(e) => println!("Invalid status code: {}", e),
    }

    // Handle invalid status code
    match HttpStatus::try_from(999) {
        Ok(status) => println!("Code 999 -> Status: {:?}", status),
        Err(e) => println!("Invalid status code: {}", e),
    }

    // Convert integer to enum using from_repr method
    if let Some(status) = HttpStatus::from_repr(200) {
        println!("Code 200 -> Status: {:?}", status);
    }

    // Demonstrate reference conversion (doesn't consume the enum)
    let status = HttpStatus::Ok;
    let code_from_ref: u16 = (&status).into();
    println!("Reference conversion: {:?} -> {} (enum still available)", status, code_from_ref);

    println!("\n=== Example 2: Priority Levels ===\n");

    // Working with signed integers
    let priority = Priority::Critical;
    let priority_value: i8 = priority.into();
    println!("Priority: {:?} -> Value: {}", priority, priority_value);

    // Convert negative value
    let low_priority = Priority::try_from(-1).unwrap();
    println!("Value -1 -> Priority: {:?}", low_priority);

    println!("\n=== Example 3: Implicit Discriminants ===\n");

    // Enum with implicit discriminants (0, 1, 2)
    println!("Red = {}", Color::Red.as_repr());
    println!("Green = {}", Color::Green.as_repr());
    println!("Blue = {}", Color::Blue.as_repr());

    let color = Color::from_repr(1).unwrap();
    println!("Value 1 -> Color: {:?}", color);

    println!("\n=== Example 4: Const Contexts ===\n");

    // The generated methods are const, so they work in const contexts
    const OK_CODE: u16 = HttpStatus::Ok.as_repr();
    const RED_VALUE: u8 = Color::Red.as_repr();

    println!("OK_CODE (const): {}", OK_CODE);
    println!("RED_VALUE (const): {}", RED_VALUE);

    const MAYBE_GREEN: Option<Color> = Color::from_repr(1);
    if let Some(color) = MAYBE_GREEN {
        println!("Const conversion: {:?}", color);
    }

    println!("\n=== Example 5: Error Handling ===\n");

    // Demonstrate error type
    let result = HttpStatus::try_from(123);
    match result {
        Ok(status) => println!("Valid status: {:?}", status),
        Err(err) => {
            println!("Error: {}", err);
            println!("Invalid value was: {}", err.0);
        }
    }

    println!("\n=== Example 6: Complex Discriminant Expressions ===\n");

    // Demonstrate const expressions as discriminants
    println!("ErrorCode discriminants:");
    println!("  Success = {}", ErrorCode::Success.as_repr());
    println!("  InvalidInput = {}", ErrorCode::InvalidInput.as_repr());
    println!("  BadRequest = {} (from ERROR_BASE)", ErrorCode::BadRequest.as_repr());
    println!("  Forbidden = {}", ErrorCode::Forbidden.as_repr());
    println!("  InternalError = {} (from SERVER_ERROR_BASE)", ErrorCode::InternalError.as_repr());

    // Verify const expression evaluation
    assert_eq!(ErrorCode::BadRequest.as_repr(), ERROR_BASE);
    assert_eq!(ErrorCode::InternalError.as_repr(), SERVER_ERROR_BASE);

    // Conversion works correctly
    assert_eq!(ErrorCode::try_from(ERROR_BASE).unwrap(), ErrorCode::BadRequest);
    assert_eq!(ErrorCode::try_from(SERVER_ERROR_BASE).unwrap(), ErrorCode::InternalError);

    println!("\nConst expressions work correctly!");

    println!("\n=== Example 7: Round-trip Conversion ===\n");

    // Demonstrate round-trip conversion
    let original = Priority::High;
    let as_int: i8 = original.into();
    let back_to_enum = Priority::try_from(as_int).unwrap();

    println!("Original: {:?}", original);
    println!("As integer: {}", as_int);
    println!("Back to enum: {:?}", back_to_enum);
    println!("Round-trip successful: {}", original == back_to_enum);

    println!("\n=== Example 8: Reference Conversions ===\n");

    // Reference conversions allow multiple uses without consuming the enum
    let color = Color::Green;

    // Convert via reference multiple times
    let value1: u8 = (&color).into();
    let value2: u8 = (&color).into();
    let value3 = u8::from(&color);

    println!("Color: {:?}", color);
    println!("  Converted 3 times: {}, {}, {}", value1, value2, value3);
    println!("  Enum still available after conversions!");

    // Useful in functions that need to convert without taking ownership
    fn log_status(status: &HttpStatus) {
        let code: u16 = status.into();
        println!("Logging status {:?} with code {}", status, code);
    }

    let my_status = HttpStatus::Created;
    log_status(&my_status);
    log_status(&my_status); // Can use again because we only passed a reference

    println!("\nReference conversions prevent unnecessary clones!");

    println!("\n=== Example 9: TryFrom with References ===\n");

    // TryFrom also works with integer references
    let code = 404u16;
    let code_ref = &code;

    // Convert from integer reference to enum
    let status_result = HttpStatus::try_from(code_ref);
    match status_result {
        Ok(status) => println!("Code {} -> Status: {:?}", code, status),
        Err(e) => println!("Invalid code: {}", e),
    }

    // Can use the original value after reference conversion
    println!("Original code value still available: {}", code);

    // Works with both valid and invalid values
    let valid_codes = [200u16, 201, 404, 500];
    for code in &valid_codes {
        // Using reference to avoid copying
        match HttpStatus::try_from(code) {
            Ok(status) => println!("  {} is {:?}", code, status),
            Err(_) => println!("  {} is not a valid HttpStatus", code),
        }
    }

    // Error handling with references
    let invalid_code = 999u16;
    let result = HttpStatus::try_from(&invalid_code);
    if let Err(e) = result {
        println!("\nError: {} (code was {})", e, invalid_code);
    }
}
