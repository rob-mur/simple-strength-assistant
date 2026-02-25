# Coding Conventions

**Analysis Date:** 2026-02-25

## Naming Patterns

**Files:**
- Rust: `snake_case.rs` (e.g., `workout_state.rs`, `file_system.rs`, `db_tests.rs`)
- JavaScript: `kebab-case.js` (e.g., `db-module.js`, `file-handle-storage.js`)
- Test files: Module name + `_tests.rs` suffix (e.g., `db_tests.rs`, `file_system_tests.rs`)

**Functions:**
- Rust: `snake_case` (e.g., `validate_weight`, `create_session`, `prompt_for_file`)
- JavaScript: `camelCase` (e.g., `executeQuery`, `initDatabase`, `ensureSQLLoaded`)

**Variables:**
- Rust: `snake_case` (e.g., `min_weight`, `session_id`, `completed_sets`)
- JavaScript: `camelCase` (e.g., `fileData`, `uint8Array`)

**Types:**
- Rust structs/enums: `PascalCase` (e.g., `WorkoutState`, `ExerciseMetadata`, `ValidationError`)
- Rust enum variants: `PascalCase` (e.g., `NotInitialized`, `Weighted`, `Bodyweight`)

## Code Style

**Formatting:**
- Rust: Standard `rustfmt` (edition 2024)
- JavaScript: No formatter configured (inconsistent style observed)
- Indentation: 4 spaces for Rust, 4 spaces for JavaScript

**Linting:**
- Rust: Standard `cargo clippy` with `#[allow(dead_code)]` and `#[allow(unused_imports)]` used selectively
- JavaScript: No linter configured
- Commit messages: Enforced via commitlint (conventional commits)

## Import Organization

**Order (Rust):**
1. Crate-local imports (`crate::models`, `crate::state`)
2. External crate imports (`use serde::{Deserialize, Serialize}`)
3. Standard library imports (`use std::cell::RefCell`)

**Path Aliases:**
- No path aliases configured
- All imports use explicit relative or crate-qualified paths

## Error Handling

**Patterns:**
- Use `thiserror` for custom error types (e.g., `DatabaseError`, `FileSystemError`, `ValidationError`)
- All errors implement `std::error::Error` and `Display`
- Error propagation via `?` operator
- Async errors returned as `Result<T, String>` in state manager
- JavaScript errors converted to `DatabaseError::JsError` via `From<JsValue>`

**Example:**
```rust
#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Failed to initialize database: {0}")]
    InitializationError(String),

    #[error("Failed to execute query: {0}")]
    QueryError(String),
}
```

**User-facing error handling:**
- Error display via `fmt::Display` with descriptive messages
- Errors logged with `log::error!` and `web_sys::console::error_1`
- State transitions to `InitializationState::Error` on failure

## Logging

**Framework:** Standard `log` crate + `web_sys::console`

**Patterns:**
- Use `log::error!`, `log::warn!`, `log::info!` for Rust logging
- Use `web_sys::console::log_1`, `web_sys::console::error_1`, `web_sys::console::warn_1` for browser console
- JavaScript uses `console.log`, `console.error`
- Prefix log messages with context (e.g., `"[DB Init]"`, `"[FileSystem]"`, `"[Workout]"`)

**When to log:**
- Initialization steps and state transitions
- Error conditions (always)
- Warnings for recoverable issues
- Info for significant operations (database setup, file operations)

## Comments

**When to Comment:**
- Module-level documentation (triple-slash `///`)
- Public API documentation with descriptions, arguments, returns
- Complex logic requiring explanation
- Business rule constants (e.g., `DEFAULT_WEIGHTED_REPS`, `RPE_THRESHOLD_HIGH`)

**Documentation Style:**
- Rust: Rustdoc-style comments (`///`) for public items
- Clear section headers in implementation files
- Type-level and function-level documentation mandatory for public APIs

**Example:**
```rust
/// Validates that a weight is at or above the minimum and is a valid multiple of the increment.
///
/// # Arguments
/// * `weight` - The weight to validate
/// * `min_weight` - The minimum allowed weight
/// * `increment` - The weight increment (e.g., 2.5kg)
///
/// # Returns
/// `Ok(())` if valid, otherwise a `ValidationError`
pub fn validate_weight(weight: f32, min_weight: f32, increment: f32) -> Result<(), ValidationError>
```

## Function Design

**Size:** Functions are generally concise (10-50 lines), with some larger UI components (100+ lines)

**Parameters:**
- Prefer borrowed references (`&str`, `&CompletedSet`) over owned values
- Use `&self` for methods that don't mutate
- Use `async` for I/O operations

**Return Values:**
- Use `Result<T, E>` for fallible operations
- Use `Option<T>` for nullable values
- Async functions return `impl Future<Output = Result<T, E>>`

## Module Design

**Exports:**
- Use `pub use` in `mod.rs` to re-export commonly used types
- Mark internal implementation details as private
- Use `#[allow(dead_code)]` for exported types not yet used

**Barrel Files:**
- Each major module has a `mod.rs` that declares submodules and re-exports
- Pattern: `pub use submodule::{Type1, Type2}`

**Example (`src/models/mod.rs`):**
```rust
pub mod exercise;
pub mod set;
pub mod validation;

#[allow(unused_imports)]
pub use exercise::{ExerciseMetadata, SetTypeConfig};
#[allow(unused_imports)]
pub use set::{CompletedSet, SetType};
```

## Constants

**Naming:** `SCREAMING_SNAKE_CASE`

**Location:** Defined at module or function level near usage

**Examples:**
- `MAX_EXERCISE_NAME_LENGTH: usize = 100`
- `DEFAULT_WEIGHTED_REPS: u32 = 8`
- `RPE_THRESHOLD_HIGH: f32 = 8.0`
- `MAX_FILE_SIZE: usize = 100 * 1024 * 1024`
- `SQLITE_MAGIC_NUMBER: &[u8] = b"SQLite format 3\0"`

## Type Safety

**Pattern matching:**
- Exhaustive enum matching required
- No catch-all wildcards unless genuinely needed
- Pattern match on both data type and configuration to enforce consistency

**Example:**
```rust
match (&set.set_type, &exercise.set_type_config) {
    (SetType::Weighted { weight }, SetTypeConfig::Weighted { min_weight, increment }) => {
        validate_weight(*weight, *min_weight, *increment)?;
    }
    (SetType::Bodyweight, SetTypeConfig::Bodyweight) => {
        // Valid combination
    }
    _ => {
        // Type mismatch cases
    }
}
```

## Async Patterns

**Concurrency:**
- Use `async fn` for all I/O operations
- Database operations are all async
- File system operations are all async
- Use `await` at call sites

**Error handling in async:**
- Return `Result<T, ErrorType>` from async functions
- Map errors at boundaries (JS errors → Rust errors)

---

*Convention analysis: 2026-02-25*
