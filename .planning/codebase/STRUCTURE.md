# Directory Structure

**Analysis Date:** 2026-02-25

## Root Layout

```
.
├── src/                    # Rust source code
├── public/                 # Static assets and JS bridge modules
├── .github/                # GitHub Actions workflows
├── .claude/                # GSD workflow configuration
├── Cargo.toml              # Rust dependencies
├── package.json            # Node dependencies (CSS tooling)
├── Dioxus.toml             # Dioxus framework config
├── devenv.nix              # Development environment
├── tailwind.config.js      # Tailwind CSS config
├── postcss.config.js       # PostCSS config
├── index.html              # HTML entry point
└── README.md               # Project documentation
```

## Source Directory (`src/`)

```
src/
├── main.rs                 # Application entry point
├── app.rs                  # Root UI component and initialization
├── models/                 # Domain types and validation
│   ├── mod.rs              # Module exports
│   ├── exercise.rs         # Exercise metadata types
│   ├── set.rs              # Set types (Weighted, Bodyweight, etc.)
│   └── validation.rs       # Validation functions and tests
└── state/                  # State management and data access
    ├── mod.rs              # Module exports
    ├── workout_state.rs    # Application state and session management
    ├── db.rs               # Database interface (WASM FFI to sql.js)
    ├── db_tests.rs         # Database integration tests
    ├── file_system.rs      # File System Access API wrapper
    └── file_system_tests.rs # File system integration tests
```

## Public Assets (`public/`)

```
public/
├── db-module.js            # SQLite WASM bridge (sql.js wrapper)
├── file-handle-storage.js  # File System Access API utilities
├── sql-wasm.js             # sql.js library
├── sql-wasm.wasm           # SQLite compiled to WASM
├── service-worker.js       # PWA offline support
├── vercel.json             # Vercel deployment config
└── styles.css              # Generated Tailwind CSS (build artifact)
```

## CI/CD (`./github/workflows/`)

```
.github/workflows/
├── deploy.yml              # Production deployment to Vercel
└── ci.yml                  # Continuous integration checks
```

## Key Locations

**Entry points:**
- `src/main.rs` - Application bootstrap
- `index.html` - HTML shell for WASM app
- `src/app.rs` - Root React-like component

**Business logic:**
- `src/state/workout_state.rs` - Core application state (412 lines)
- `src/models/validation.rs` - Validation rules (280 lines of tests)

**Data access:**
- `src/state/db.rs` - Database operations (250 lines)
- `src/state/file_system.rs` - File I/O (334 lines)
- `public/db-module.js` - JS bridge for SQLite (194 lines)
- `public/file-handle-storage.js` - JS bridge for File System Access API

**Tests:**
- `src/state/db_tests.rs` - Database tests (278 lines, 17 tests)
- `src/state/file_system_tests.rs` - File system tests (227 lines, 12 tests)
- Inline tests in `src/models/validation.rs` (embedded with implementation)

**Configuration:**
- `Cargo.toml` - Rust dependencies and metadata
- `Dioxus.toml` - Dioxus app config (app name, assets, web settings)
- `tailwind.config.js` - Tailwind CSS customization
- `devenv.nix` - Development environment definition
- `.releaserc.json` - Semantic release configuration

## Naming Conventions

**Rust files:**
- `snake_case.rs` for all modules
- `*_tests.rs` suffix for test modules (co-located with implementation)
- `mod.rs` for module declarations and re-exports

**JavaScript files:**
- `kebab-case.js` for all modules
- Descriptive names: `db-module.js`, `file-handle-storage.js`

**Directories:**
- `snake_case` for all directories
- Plural names for collections: `models/`, `workflows/`
- Singular names for single-purpose: `state/`, `public/`

## Module Organization

**Barrel files (`mod.rs`):**
- Declare submodules with `pub mod submodule;`
- Re-export commonly used types with `pub use submodule::Type;`
- Allow `#[allow(unused_imports)]` for exports not yet used

**Example from `src/models/mod.rs`:**
```rust
pub mod exercise;
pub mod set;
pub mod validation;

#[allow(unused_imports)]
pub use exercise::{ExerciseMetadata, SetTypeConfig};
#[allow(unused_imports)]
pub use set::{CompletedSet, SetType};
```

## Where to Add New Code

**New UI components:**
- Add to `src/app.rs` or create `src/components/` directory if splitting

**New domain types:**
- Add to `src/models/` directory
- Update `src/models/mod.rs` to export new types

**New business logic:**
- Add methods to `WorkoutStateManager` in `src/state/workout_state.rs`
- Keep state manager stateless (operate on `WorkoutState` reference)

**New database operations:**
- Add methods to `Database` in `src/state/db.rs`
- Follow async pattern with `Result<T, DatabaseError>`

**New validation rules:**
- Add functions to `src/models/validation.rs`
- Include inline tests in `#[cfg(test)]` module

**New browser API integrations:**
- Add JavaScript bridge module in `public/` directory
- Add Rust wrapper in `src/state/` directory
- Follow `Database`/`FileSystemManager` pattern

**Tests:**
- Co-locate with implementation: `foo.rs` → `foo_tests.rs` in same directory
- Use `#[cfg(test)]` modules for unit tests
- Use `wasm-bindgen-test` for browser integration tests

---

*Structure analysis: 2026-02-25*
