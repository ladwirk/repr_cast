use repr_cast::repr_cast;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u8)]
enum Status {
    Pending = 0,
    Active = 1,
    Completed = 2,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(i32)]
enum Priority {
    Low = -1,
    Normal = 0,
    High = 1,
    Critical = 100,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr_cast(u16)]
enum ImplicitDiscriminant {
    First,
    Second,
    Third,
}

#[test]
fn test_enum_to_int_conversion() {
    // Test From trait
    assert_eq!(u8::from(Status::Pending), 0);
    assert_eq!(u8::from(Status::Active), 1);
    assert_eq!(u8::from(Status::Completed), 2);

    // Test as_repr method
    assert_eq!(Status::Pending.as_repr(), 0);
    assert_eq!(Status::Active.as_repr(), 1);
    assert_eq!(Status::Completed.as_repr(), 2);
}

#[test]
fn test_int_to_enum_conversion() {
    // Test TryFrom trait with valid values
    assert_eq!(Status::try_from(0).unwrap(), Status::Pending);
    assert_eq!(Status::try_from(1).unwrap(), Status::Active);
    assert_eq!(Status::try_from(2).unwrap(), Status::Completed);

    // Test TryFrom trait with invalid values
    assert!(Status::try_from(3).is_err());
    assert!(Status::try_from(255).is_err());

    // Test from_repr method
    assert_eq!(Status::from_repr(0), Some(Status::Pending));
    assert_eq!(Status::from_repr(1), Some(Status::Active));
    assert_eq!(Status::from_repr(2), Some(Status::Completed));
    assert_eq!(Status::from_repr(3), None);
}

#[test]
fn test_signed_integers() {
    assert_eq!(i32::from(Priority::Low), -1);
    assert_eq!(i32::from(Priority::Normal), 0);
    assert_eq!(i32::from(Priority::High), 1);
    assert_eq!(i32::from(Priority::Critical), 100);

    assert_eq!(Priority::try_from(-1).unwrap(), Priority::Low);
    assert_eq!(Priority::try_from(0).unwrap(), Priority::Normal);
    assert_eq!(Priority::try_from(1).unwrap(), Priority::High);
    assert_eq!(Priority::try_from(100).unwrap(), Priority::Critical);

    assert!(Priority::try_from(2).is_err());
    assert!(Priority::try_from(-2).is_err());
}

#[test]
fn test_implicit_discriminant() {
    assert_eq!(u16::from(ImplicitDiscriminant::First), 0);
    assert_eq!(u16::from(ImplicitDiscriminant::Second), 1);
    assert_eq!(u16::from(ImplicitDiscriminant::Third), 2);

    assert_eq!(
        ImplicitDiscriminant::try_from(0).unwrap(),
        ImplicitDiscriminant::First
    );
    assert_eq!(
        ImplicitDiscriminant::try_from(1).unwrap(),
        ImplicitDiscriminant::Second
    );
    assert_eq!(
        ImplicitDiscriminant::try_from(2).unwrap(),
        ImplicitDiscriminant::Third
    );
}

#[test]
fn test_const_functions() {
    // Test that from_repr and as_repr are const functions
    const PENDING: Option<Status> = Status::from_repr(0);
    assert_eq!(PENDING, Some(Status::Pending));

    const PENDING_VALUE: u8 = Status::Pending.as_repr();
    assert_eq!(PENDING_VALUE, 0);
}

#[test]
fn test_error_type() {
    let result = Status::try_from(99);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.0, 99);

    // Test Display implementation
    let error_msg = format!("{}", err);
    assert!(error_msg.contains("unknown Status variant"));
    assert!(error_msg.contains("99"));
}

// Test that the macro properly implements PartialEq for the enum
#[test]
fn test_enum_equality() {
    assert_eq!(Status::Pending, Status::Pending);
    assert_ne!(Status::Pending, Status::Active);

    let status1 = Status::from_repr(1).unwrap();
    let status2 = Status::Active;
    assert_eq!(status1, status2);
}

// Test round-trip conversion
#[test]
fn test_round_trip_conversion() {
    for value in [Status::Pending, Status::Active, Status::Completed] {
        let int_value = u8::from(value);
        let back_to_enum = Status::try_from(int_value).unwrap();
        assert_eq!(value, back_to_enum);
    }
}

// Test From trait with references
#[test]
fn test_from_reference() {
    let status = Status::Active;
    let status_ref = &status;

    // Test From<&Status> for u8
    let value: u8 = status_ref.into();
    assert_eq!(value, 1);

    // Test with explicit From::from
    let value2 = u8::from(status_ref);
    assert_eq!(value2, 1);

    // Test with all variants
    assert_eq!(u8::from(&Status::Pending), 0);
    assert_eq!(u8::from(&Status::Active), 1);
    assert_eq!(u8::from(&Status::Completed), 2);
}

#[test]
fn test_reference_vs_owned_conversion() {
    let status = Status::Active;

    // Both should give the same result
    let from_owned = u8::from(status);
    let from_ref = u8::from(&status);

    assert_eq!(from_owned, from_ref);
    assert_eq!(from_ref, 1);
}

#[test]
fn test_reference_conversion_signed() {
    let priority = Priority::Low;
    let priority_ref = &priority;

    let value: i32 = priority_ref.into();
    assert_eq!(value, -1);

    // Test with all variants
    assert_eq!(i32::from(&Priority::Low), -1);
    assert_eq!(i32::from(&Priority::Normal), 0);
    assert_eq!(i32::from(&Priority::High), 1);
    assert_eq!(i32::from(&Priority::Critical), 100);
}

#[test]
fn test_reference_conversion_implicit_discriminants() {
    let color = ImplicitDiscriminant::Second;
    let color_ref = &color;

    assert_eq!(u16::from(color_ref), 1);
    assert_eq!(u16::from(&ImplicitDiscriminant::First), 0);
    assert_eq!(u16::from(&ImplicitDiscriminant::Second), 1);
    assert_eq!(u16::from(&ImplicitDiscriminant::Third), 2);
}

// Test TryFrom trait with integer references
#[test]
fn test_try_from_reference() {
    let value: u8 = 1;
    let value_ref = &value;

    // Test TryFrom<&u8> for Status
    let status: Status = value_ref.try_into().unwrap();
    assert_eq!(status, Status::Active);

    // Test with explicit TryFrom::try_from
    let status2 = Status::try_from(value_ref).unwrap();
    assert_eq!(status2, Status::Active);

    // Test with all variants
    assert_eq!(Status::try_from(&0u8).unwrap(), Status::Pending);
    assert_eq!(Status::try_from(&1u8).unwrap(), Status::Active);
    assert_eq!(Status::try_from(&2u8).unwrap(), Status::Completed);

    // Test with invalid value
    let invalid_value = 99u8;
    assert!(Status::try_from(&invalid_value).is_err());
}

#[test]
fn test_try_from_reference_vs_owned() {
    let value = 1u8;

    // Both should give the same result
    let from_owned = Status::try_from(value).unwrap();
    let from_ref = Status::try_from(&value).unwrap();

    assert_eq!(from_owned, from_ref);
    assert_eq!(from_ref, Status::Active);
}

#[test]
fn test_try_from_reference_signed() {
    let value: i32 = -1;
    let value_ref = &value;

    let priority: Priority = value_ref.try_into().unwrap();
    assert_eq!(priority, Priority::Low);

    // Test with all variants
    assert_eq!(Priority::try_from(&(-1i32)).unwrap(), Priority::Low);
    assert_eq!(Priority::try_from(&0i32).unwrap(), Priority::Normal);
    assert_eq!(Priority::try_from(&1i32).unwrap(), Priority::High);
    assert_eq!(Priority::try_from(&100i32).unwrap(), Priority::Critical);

    // Test with invalid value
    assert!(Priority::try_from(&50i32).is_err());
}

#[test]
fn test_try_from_reference_error() {
    let invalid_value = 99u8;
    let invalid_ref = &invalid_value;

    let result = Status::try_from(invalid_ref);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.0, 99);
}

#[test]
fn test_try_from_reference_implicit_discriminants() {
    let value: u16 = 1;
    let value_ref = &value;

    assert_eq!(
        ImplicitDiscriminant::try_from(value_ref).unwrap(),
        ImplicitDiscriminant::Second
    );
    assert_eq!(
        ImplicitDiscriminant::try_from(&0u16).unwrap(),
        ImplicitDiscriminant::First
    );
    assert_eq!(
        ImplicitDiscriminant::try_from(&2u16).unwrap(),
        ImplicitDiscriminant::Third
    );

    // Test with invalid value
    assert!(ImplicitDiscriminant::try_from(&5u16).is_err());
}
