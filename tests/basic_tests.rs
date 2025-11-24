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
