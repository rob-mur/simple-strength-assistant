# Architecture

**Analysis Date:** 2026-02-25

## Pattern Overview

**Overall:** Component-based reactive UI with layered state management (Dioxus web application)

**Key Characteristics:**
- Rust compiled to WebAssembly for web deployment
- Reactive UI with component-based architecture (Dioxus framework)
- Client-side SQLite database via sql.js WASM
- Browser File System Access API with fallback to LocalStorage/IndexedDB
- Pure client-side application - no backend server

## Layers

**Presentation Layer (UI Components):**
- Purpose: Render UI and handle user interactions
- Location: `src/app.rs`
- Contains: Dioxus components (`App`, `WorkoutInterface`, `StartSessionView`, `ActiveSession`)
- Depends on: State layer (`WorkoutState`, `WorkoutStateManager`)
- Used by: Entry point (`src/main.rs`)

**State Management Layer:**
- Purpose: Coordinate application state, session management, and business logic
- Location: `src/state/workout_state.rs`
- Contains: `WorkoutState` (reactive state container), `WorkoutStateManager` (business operations), `WorkoutSession`, `PredictedParameters`
- Depends on: Data layer (Database, FileSystemManager), Models layer
- Used by: Presentation layer components

**Data Layer:**
- Purpose: Abstract persistence and file operations
- Location: `src/state/db.rs`, `src/state/file_system.rs`
- Contains: `Database` (SQLite operations via WASM FFI), `FileSystemManager` (file I/O with fallback)
- Depends on: JavaScript bridge modules (`/public/db-module.js`, `/public/file-handle-storage.js`)
- Used by: State management layer

**Models Layer:**
- Purpose: Define core domain types and validation rules
- Location: `src/models/`
- Contains: `ExerciseMetadata`, `CompletedSet`, `SetType`, `SetTypeConfig`, validation functions
- Depends on: Nothing (pure domain logic)
- Used by: All other layers

**JavaScript Bridge Layer:**
- Purpose: Interface between WASM and browser APIs
- Location: `/public/db-module.js`, `/public/file-handle-storage.js`
- Contains: SQL.js initialization, File System Access API wrappers, IndexedDB persistence
- Depends on: Browser APIs, sql.js library
- Used by: Data layer via WASM bindings

## Data Flow

**Application Initialization:**

1. `src/main.rs` launches `App` component via `dioxus::launch()`
2. `App` creates `WorkoutState` context provider
3. `use_effect` hook triggers `WorkoutStateManager::setup_database()`
4. Database setup flow:
   - Create `FileSystemManager` and check for cached file handle
   - If no cached handle: prompt user for file selection (or use fallback storage)
   - Read existing file data or create new database
   - Initialize `Database` with file data
   - Store database and file manager in `WorkoutState`
   - Set initialization state to `Ready`

**Starting a Workout Session:**

1. User fills form in `StartSessionView` component
2. On submit: `WorkoutStateManager::start_session()` called with `ExerciseMetadata`
3. State manager saves exercise to database
4. State manager creates session in database (returns session_id)
5. State manager calculates initial predictions
6. New `WorkoutSession` stored in `WorkoutState.current_session`
7. UI switches to `ActiveSession` component

**Logging a Set:**

1. User enters set data in `ActiveSession` component
2. On submit: `WorkoutStateManager::log_set()` called with `CompletedSet`
3. Validation runs via `validate_completed_set()`
4. Set inserted into database
5. Set added to session's `completed_sets` vector
6. Next predictions calculated based on RPE and performance
7. Database auto-saved to file
8. UI updates to show new set and updated predictions

**Completing a Session:**

1. User clicks "Complete Session" button
2. `WorkoutStateManager::complete_session()` called
3. Session marked complete in database
4. Database saved to file
5. `WorkoutState.current_session` cleared
6. UI returns to `StartSessionView`

**State Management:**
- Reactive state via Dioxus signals and context
- `WorkoutState` uses interior mutability pattern (`Rc<RefCell<WorkoutStateInner>>`)
- State updates trigger UI re-renders automatically
- Database persisted to file after each set (auto-save) and session completion

## Key Abstractions

**WorkoutState:**
- Purpose: Central reactive state container for application
- Examples: `src/state/workout_state.rs` (lines 40-115)
- Pattern: Interior mutability with `Rc<RefCell<T>>` for shared ownership across components

**WorkoutStateManager:**
- Purpose: Stateless coordinator for business operations
- Examples: `src/state/workout_state.rs` (lines 117-412)
- Pattern: Static methods operating on `WorkoutState` reference

**Database:**
- Purpose: Type-safe wrapper around sql.js WASM interface
- Examples: `src/state/db.rs`
- Pattern: Async FFI bridge to JavaScript with strongly-typed Rust API

**FileSystemManager:**
- Purpose: Abstract file persistence with graceful degradation
- Examples: `src/state/file_system.rs`
- Pattern: Try File System Access API, fallback to LocalStorage if unsupported

**SetType/SetTypeConfig:**
- Purpose: Type-safe distinction between exercise configuration and actual sets
- Examples: `src/models/set.rs`, `src/models/exercise.rs`
- Pattern: Enum-based sum types for mutually exclusive states

## Entry Points

**Main Entry:**
- Location: `src/main.rs`
- Triggers: Application load
- Responsibilities: Launch Dioxus framework with `App` component

**App Component:**
- Location: `src/app.rs` (lines 6-144)
- Triggers: Mounted by Dioxus runtime
- Responsibilities: Initialize workout state, set up database, render initialization flow

**Database Initialization:**
- Location: `src/state/workout_state.rs` (lines 120-218)
- Triggers: `use_effect` hook on `App` mount
- Responsibilities: Set up file system access, load or create database, transition to Ready state

## Error Handling

**Strategy:** Result-based error propagation with user-facing error states

**Patterns:**
- Custom error types: `DatabaseError`, `FileSystemError` (using `thiserror` crate)
- Result types returned from all fallible operations
- Errors converted to strings at state manager boundary
- UI displays errors via `InitializationState::Error` with retry mechanism
- Logging via `log` crate and `web_sys::console`
- Database auto-save failures logged but don't fail the operation

## Cross-Cutting Concerns

**Logging:**
- Console logging via `web_sys::console::log_1()` for debugging
- Structured logging via `log` crate macros (`log::error!`, `log::warn!`)

**Validation:**
- Centralized in `src/models/validation.rs`
- Validation functions return `Result<(), ValidationError>`
- Validation called before database operations in `WorkoutStateManager::log_set()`
- UI-level validation for exercise names in `src/app.rs` (lines 163-174)

**Authentication:**
- Not applicable (client-side only application, no server)

---

*Architecture analysis: 2026-02-25*
