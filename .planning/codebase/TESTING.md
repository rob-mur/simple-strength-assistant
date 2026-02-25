# Testing

**Analysis Date:** 2026-02-25

## Test Framework

**Primary:**
- wasm-bindgen-test - Browser-based testing for WASM
- Requires headless Chrome for test execution

**Command:**
```bash
wasm-pack test --headless --chrome
```

## Test File Organization

**Structure:**
- Tests co-located with implementation
- Naming: `module_tests.rs` for integration tests (e.g., `db_tests.rs`, `file_system_tests.rs`)
- Inline tests: `#[cfg(test)]` modules within source files for unit tests

**Example locations:**
- `src/state/db_tests.rs` - Database integration tests (278 lines, 17 tests)
- `src/state/file_system_tests.rs` - File system integration tests (227 lines, 12 tests)
- `src/models/validation.rs` - Inline validation tests (280 lines in test module)

## Test Structure

**Setup:**
```rust
#![cfg(test)]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_name() {
    // Arrange
    let db = Database::new().await.expect("Failed to init db");

    // Act
    let result = db.some_operation().await;

    // Assert
    assert!(result.is_ok());
}
```

**Key attributes:**
- `#[wasm_bindgen_test]` - Marks test for browser execution
- `wasm_bindgen_test_configure!(run_in_browser)` - Global test configuration
- Tests are async when testing async functions

## Mocking

**Approach:**
- No mocking framework used
- Tests run against real sql.js and LocalStorage APIs in browser
- Integration tests rather than unit tests with mocks

**Rationale:**
- Browser APIs (sql.js, LocalStorage) are lightweight and fast
- Real API testing catches more bugs than mocking
- WASM test environment provides isolated browser context

## Fixtures and Factories

**Pattern:**
- Inline test data creation
- Helper functions for common test data

**Example:**
```rust
fn create_test_exercise() -> ExerciseMetadata {
    ExerciseMetadata {
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        default_reps: 8,
    }
}
```

**Common test values:**
- Exercise names: "Bench Press", "Squat", "Pull-ups"
- Weights: 60.0, 100.0 (with increment 2.5, min 20.0)
- Reps: 8-12
- RPE: 7.0-9.0

## Coverage

**Current coverage:**
- Database operations: Comprehensive (17 tests covering init, queries, transactions)
- File system operations: Comprehensive (12 tests covering file I/O, permissions, fallback)
- Validation: Comprehensive (280 lines of tests for all validation rules)
- UI: Not tested (manual testing only)
- State management: Not tested (no state machine tests)

**Gaps:**
- No tests for `WorkoutStateManager` business logic
- No tests for UI components
- No tests for error recovery and state transitions
- No tests for concurrent operations

## Test Types

**Integration tests (primary):**
- Test full stack from Rust through WASM FFI to JavaScript
- Examples: Database CRUD operations, file system I/O, validation chains

**Unit tests (limited):**
- Inline tests in implementation files
- Examples: Pure functions in `validation.rs`

**No end-to-end tests:**
- Would require full browser automation
- Currently relying on manual testing for UI workflows

## Common Patterns

**Async testing:**
```rust
#[wasm_bindgen_test]
async fn test_async_operation() {
    let result = some_async_function().await;
    assert!(result.is_ok());
}
```

**Error path testing:**
```rust
#[wasm_bindgen_test]
async fn test_validation_failure() {
    let result = validate_weight(-1.0, 0.0, 2.5);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Weight must be at least 0.0"
    );
}
```

**Setup and cleanup:**
```rust
#[wasm_bindgen_test]
async fn test_with_cleanup() {
    // Setup
    clear_local_storage();

    // Test
    let result = operation_using_storage().await;

    // Cleanup
    clear_local_storage();

    // Assert
    assert!(result.is_ok());
}
```

## Test Data Patterns

**SQLite magic number validation:**
```rust
const SQLITE_MAGIC: &[u8] = b"SQLite format 3\0";
assert_eq!(&file_data[0..16], SQLITE_MAGIC);
```

**LocalStorage key conventions:**
```rust
const STORAGE_KEY: &str = "workout_db_data";
const HANDLE_STORE_NAME: &str = "fileHandles";
```

**Realistic workout data:**
```rust
let set = CompletedSet {
    reps: 10,
    rpe: 8.0,
    set_type: SetType::Weighted { weight: 60.0 },
    notes: None,
};
```

## Running Tests

**All tests:**
```bash
wasm-pack test --headless --chrome
```

**Specific test file:**
```bash
wasm-pack test --headless --chrome --test db_tests
```

**Watch mode:**
- Not configured
- Use file watcher with test runner externally

## Test Statistics

**Total tests:** 29+ tests across codebase
**Total test code:** ~785 lines
- `db_tests.rs`: 278 lines, 17 tests
- `file_system_tests.rs`: 227 lines, 12 tests
- `validation.rs`: 280 lines in test module

**Pass rate:** Tests are passing (based on codebase health)

---

*Testing analysis: 2026-02-25*
