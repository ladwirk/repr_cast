use repr_cast::repr_cast;
use std::convert::TryFrom;

// Test with const expressions
const BASE: u8 = 10;
const OFFSET: u8 = 5;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u8)]
enum WithConsts {
    First,          // 0
    Second = BASE,  // 10
    Third,          // 11
    Fourth = BASE + OFFSET, // 15
    Fifth,          // 16
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u16)]
enum MixedExpressions {
    Zero,           // 0
    One,            // 1
    Ten = 10,       // 10
    Eleven,         // 11
    Hundred = 100,  // 100
    HundredOne,     // 101
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(i8)]
enum SignedMixed {
    NegTwo = -2,    // -2
    NegOne,         // -1
    Zero,           // 0
    Five = 5,       // 5
    Six,            // 6
}

#[test]
fn test_const_expressions() {
    // Test enum to int
    assert_eq!(WithConsts::First.as_repr(), 0);
    assert_eq!(WithConsts::Second.as_repr(), BASE);
    assert_eq!(WithConsts::Third.as_repr(), 11);
    assert_eq!(WithConsts::Fourth.as_repr(), BASE + OFFSET);
    assert_eq!(WithConsts::Fifth.as_repr(), 16);

    // Test int to enum
    assert_eq!(WithConsts::try_from(0).unwrap(), WithConsts::First);
    assert_eq!(WithConsts::try_from(BASE).unwrap(), WithConsts::Second);
    assert_eq!(WithConsts::try_from(11).unwrap(), WithConsts::Third);
    assert_eq!(WithConsts::try_from(BASE + OFFSET).unwrap(), WithConsts::Fourth);
    assert_eq!(WithConsts::try_from(16).unwrap(), WithConsts::Fifth);
}

#[test]
fn test_mixed_simple_expressions() {
    // Test enum to int
    assert_eq!(MixedExpressions::Zero.as_repr(), 0);
    assert_eq!(MixedExpressions::One.as_repr(), 1);
    assert_eq!(MixedExpressions::Ten.as_repr(), 10);
    assert_eq!(MixedExpressions::Eleven.as_repr(), 11);
    assert_eq!(MixedExpressions::Hundred.as_repr(), 100);
    assert_eq!(MixedExpressions::HundredOne.as_repr(), 101);

    // Test int to enum
    assert_eq!(MixedExpressions::try_from(0).unwrap(), MixedExpressions::Zero);
    assert_eq!(MixedExpressions::try_from(1).unwrap(), MixedExpressions::One);
    assert_eq!(MixedExpressions::try_from(10).unwrap(), MixedExpressions::Ten);
    assert_eq!(MixedExpressions::try_from(11).unwrap(), MixedExpressions::Eleven);
    assert_eq!(MixedExpressions::try_from(100).unwrap(), MixedExpressions::Hundred);
    assert_eq!(MixedExpressions::try_from(101).unwrap(), MixedExpressions::HundredOne);

    // Test that gaps don't convert
    assert!(MixedExpressions::try_from(2).is_err());
    assert!(MixedExpressions::try_from(50).is_err());
}

#[test]
fn test_signed_mixed() {
    // Test enum to int
    assert_eq!(SignedMixed::NegTwo.as_repr(), -2);
    assert_eq!(SignedMixed::NegOne.as_repr(), -1);
    assert_eq!(SignedMixed::Zero.as_repr(), 0);
    assert_eq!(SignedMixed::Five.as_repr(), 5);
    assert_eq!(SignedMixed::Six.as_repr(), 6);

    // Test int to enum
    assert_eq!(SignedMixed::try_from(-2).unwrap(), SignedMixed::NegTwo);
    assert_eq!(SignedMixed::try_from(-1).unwrap(), SignedMixed::NegOne);
    assert_eq!(SignedMixed::try_from(0).unwrap(), SignedMixed::Zero);
    assert_eq!(SignedMixed::try_from(5).unwrap(), SignedMixed::Five);
    assert_eq!(SignedMixed::try_from(6).unwrap(), SignedMixed::Six);

    // Test gaps
    assert!(SignedMixed::try_from(1).is_err());
    assert!(SignedMixed::try_from(-3).is_err());
}

#[test]
fn test_round_trip_with_consts() {
    for value in [0u8, BASE, 11, BASE + OFFSET, 16] {
        if let Ok(variant) = WithConsts::try_from(value) {
            let round_trip = u8::from(variant);
            assert_eq!(value, round_trip, "Round trip failed for value {}", value);
        }
    }
}

#[test]
fn test_from_repr_with_consts() {
    assert_eq!(WithConsts::from_repr(0), Some(WithConsts::First));
    assert_eq!(WithConsts::from_repr(BASE), Some(WithConsts::Second));
    assert_eq!(WithConsts::from_repr(11), Some(WithConsts::Third));
    assert_eq!(WithConsts::from_repr(BASE + OFFSET), Some(WithConsts::Fourth));
    assert_eq!(WithConsts::from_repr(16), Some(WithConsts::Fifth));
    assert_eq!(WithConsts::from_repr(99), None);
}

// Test that the generated code correctly uses the enum's discriminants
// rather than trying to compute them at macro time
#[test]
fn test_discriminants_match_rust_behavior() {
    // This test verifies that our macro generates code that matches
    // Rust's native discriminant behavior

    // For WithConsts enum, verify the actual discriminants
    assert_eq!(WithConsts::First as u8, 0);
    assert_eq!(WithConsts::Second as u8, 10);
    assert_eq!(WithConsts::Third as u8, 11);
    assert_eq!(WithConsts::Fourth as u8, 15);
    assert_eq!(WithConsts::Fifth as u8, 16);

    // Verify our as_repr matches the native cast
    assert_eq!(WithConsts::First.as_repr(), WithConsts::First as u8);
    assert_eq!(WithConsts::Second.as_repr(), WithConsts::Second as u8);
    assert_eq!(WithConsts::Third.as_repr(), WithConsts::Third as u8);
    assert_eq!(WithConsts::Fourth.as_repr(), WithConsts::Fourth as u8);
    assert_eq!(WithConsts::Fifth.as_repr(), WithConsts::Fifth as u8);
}
