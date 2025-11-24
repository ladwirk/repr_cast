# Architecture

This document describes the internal architecture of the `repr_cast` procedural macro library.

## Design Philosophy

The crate follows a **testable pipeline architecture** as recommended by [Ferrous Systems](https://ferrous-systems.com/blog/testing-proc-macros/). This separates concerns into distinct stages, each independently testable with unit tests.

## Pipeline Stages

```
TokenStream → parse → ReprEnum → expand → TokenStream
```

### 1. Parse Stage (`src/parse.rs`)

**Responsibility**: Transform raw token input into validated, structured data.

**Key Functions**:
- `parse_repr_cast()` - Main entry point for parsing
- `extract_repr_from_attrs()` - Extract repr type from existing attributes
- `calculate_discriminants()` - Compute discriminant values for variants
- `try_evaluate_expr()` - Evaluate simple integer expressions

**Validation**:
- Ensures input is an enum (not struct or union)
- Validates all variants are fieldless (unit variants)
- Filters duplicate `#[repr]` attributes
- Tracks both explicit and implicit discriminants

**Unit Tests** (14 tests):
- Simple enum parsing
- Implicit discriminants
- Signed integers
- Mixed discriminants
- Rejection of non-enums
- Rejection of enums with fields
- Attribute extraction and preservation
- Expression evaluation

### 2. Data Model (`src/repr_enum.rs`)

**Responsibility**: Define the intermediate representation between parse and expand stages.

**Key Types**:
- `ReprEnum` - Complete representation of a parsed enum
- `EnumVariant` - Represents a single enum variant
- `CalculatedDiscriminant` - Explicit or implicit discriminant values

**Features**:
- Clean separation of parsed data from token streams
- Distinguishes explicit vs implicit discriminants
- Supports future extension (e.g., generics)

**Unit Tests** (3 tests):
- Explicit discriminant token generation
- Implicit discriminant token generation
- Negative discriminant handling

### 3. Expand Stage (`src/expand.rs`)

**Responsibility**: Generate output tokens from structured data.

**Key Functions**:
- `expand_repr_cast()` - Main entry point, orchestrates all generation
- `generate_enum_definition()` - Enum with `#[repr(T)]`
- `generate_impl_methods()` - `from_repr()` and `as_repr()` methods
- `generate_from_impl()` - `From<Enum>` trait
- `generate_try_from_impl()` - `TryFrom<T>` trait
- `generate_error_type()` - Error type for failed conversions

**Unit Tests** (9 tests):
- Individual component generation (enum, methods, traits, error)
- Complete expansion
- Implicit vs explicit discriminants
- Visibility handling
- Attribute preservation
- Signed integer types

**Key Design Decision**: The `from_repr` method uses if-else chains instead of pattern matching:
```rust
if value == EnumName::Variant as ReprType {
    return Some(EnumName::Variant);
}
```
This approach enables support for complex discriminant expressions (like `BASE + OFFSET` or const references) that cannot be evaluated at macro expansion time. The enum variant is cast to the repr type at compile time, allowing any valid Rust constant expression to be used as a discriminant.

### 4. Glue Layer (`src/lib.rs`)

**Responsibility**: Minimal orchestration layer connecting stages.

**Implementation**:
- Parses proc-macro arguments
- Invokes parse stage
- Invokes expand stage
- Handles errors at each stage

**Code Size**: ~90 lines (was 230+ lines before refactoring)

## Testing Strategy

### Unit Tests (26 tests total)

Located in each module's `#[cfg(test)]` section:
- **Parse module**: 14 tests
- **Data model**: 3 tests
- **Expand module**: 9 tests

**Benefits**:
- Fast execution (no proc-macro overhead)
- Easy to debug with standard tools
- Can use `quote!` and `parse_quote!` for convenient test setup
- Clear isolation of logic stages

### Integration Tests (14 tests)

Located in `tests/`:
- `basic_tests.rs` (8 tests): Core functionality and end-to-end validation
- `complex_discriminants.rs` (6 tests): Complex const expressions and mixed discriminants
- Tests the public API
- Validates generated code compiles and works correctly

### Example Code

Located in `examples/basic_usage.rs`:
- Demonstrates real-world usage
- Serves as executable documentation
- Can be run manually for verification

## Benefits of This Architecture

1. **Testability**: Each stage can be tested independently without proc-macro complexity
2. **Debuggability**: Clear data structures make it easy to inspect intermediate states
3. **Maintainability**: Changes to one stage don't affect others
4. **Readability**: Each module has a single, clear responsibility
5. **Performance**: No runtime overhead - all stages execute at compile time

## Module Dependency Graph

```
lib.rs
  ├─> parse.rs ──> repr_enum.rs
  └─> expand.rs ──> repr_enum.rs
```

- `lib.rs` depends on `parse` and `expand`
- Both `parse` and `expand` depend on `repr_enum`
- No circular dependencies
- Clean unidirectional data flow

## Error Handling

Each stage can fail with descriptive errors:

- **Parse stage**: Syntax errors, validation failures (e.g., non-fieldless enum)
- **Expand stage**: Infallible (all validation done in parse stage)

This follows the "parse, don't validate" principle - once parsing succeeds, expansion cannot fail.

## Future Extensions

The architecture easily supports:
- **Generic enums**: Already parsed, just needs expand support
- **Custom derive attributes**: Can add to `ReprEnum` without changing pipeline
- **Additional conversions**: New generation functions in expand module
- **Optimization passes**: Insert between parse and expand stages

## Testing Tools

The crate uses:
- `syn` - Parsing Rust syntax
- `quote` - Generating Rust code
- `parse_quote!` - Convenient test setup
- Standard `#[test]` - Fast unit tests
- `trybuild` (optional) - Compile-fail tests for error messages
