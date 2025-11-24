# UI Compile-Fail Tests

This directory contains compile-fail tests for the `repr_cast` macro, using the [trybuild](https://docs.rs/trybuild) framework.

## Purpose

These tests verify that the macro produces helpful, clear error messages when given invalid input. This is an important aspect of macro usability - users should get actionable feedback when they make mistakes.

## Test Cases

| Test File | Description |
|-----------|-------------|
| `not_an_enum.rs` | Applying `repr_cast` to a struct (should fail) |
| `union_type.rs` | Applying `repr_cast` to a union (should fail) |
| `enum_with_fields.rs` | Enum with tuple variant fields |
| `enum_with_named_fields.rs` | Enum with named (struct-like) fields |
| `mixed_variants.rs` | Enum mixing unit and non-unit variants |
| `no_repr_type.rs` | Missing repr type argument |

## How It Works

Each `.rs` file is a test case that should fail to compile. The corresponding `.stderr` file contains the expected compiler error output.

When tests run:
1. trybuild compiles each `.rs` file
2. Expects compilation to fail
3. Compares the actual error output to the `.stderr` file
4. Test passes if errors match

## Running Tests

```bash
# Run UI tests
cargo test --test ui_tests

# Regenerate .stderr files (when error messages change)
TRYBUILD=overwrite cargo test --test ui_tests
```

## Adding New Tests

1. Create a new `.rs` file in this directory with invalid code
2. Run `TRYBUILD=overwrite cargo test --test ui_tests`
3. trybuild will generate the `.stderr` file automatically
4. Review the error message to ensure it's helpful
5. Commit both the `.rs` and `.stderr` files

## Best Practices

- Keep test cases focused on one error condition
- Use descriptive file names
- Ensure error messages are clear and actionable
- Test boundary conditions and common mistakes
