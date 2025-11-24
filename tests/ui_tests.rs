/// Compile-fail tests using trybuild.
///
/// These tests verify that the macro produces helpful error messages
/// when given invalid input.
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
