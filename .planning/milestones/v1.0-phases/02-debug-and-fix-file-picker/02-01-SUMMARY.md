---
phase: 02-debug-and-fix-file-picker
plan: 01
subsystem: file-picker
tags: [bugfix, user-gesture, file-system-access-api, error-handling]
dependency_graph:
  requires: [01-development-environment]
  provides: [file-picker-with-user-gesture, enhanced-error-logging]
  affects: [database-initialization, ui-flow]
tech_stack:
  added: []
  patterns: [user-gesture-activation, inline-initialization, stack-trace-capture]
key_files:
  created: []
  modified:
    - src/state/workout_state.rs
    - src/app.rs
    - src/state/file_system.rs
decisions:
  - title: "Inline initialization after file selection"
    rationale: "Eliminated fragile error message string matching by continuing database initialization inline after successful file selection, instead of recalling setup_database"
    alternatives: ["String matching on error messages", "State machine with more states"]
    selected: "Inline initialization"
  - title: "Added public setter methods to WorkoutState"
    rationale: "Required to allow UI button handler to store database and file manager after initialization"
    alternatives: ["Make inner fields public", "Friend module pattern"]
    selected: "Public setter methods (set_database, set_file_manager)"
metrics:
  duration: 4min
  tasks_completed: 3
  files_modified: 3
  commits: 3
  completed_date: 2026-02-26
---

# Phase 02 Plan 01: File Picker User Gesture Fix Summary

**One-liner:** File picker now requires user gesture via button click, with SecurityError detection and stack trace logging for WASM-JS boundary debugging

## What Was Built

Fixed file picker not appearing by ensuring showSaveFilePicker() is called from a user gesture (button click) instead of automatically during initialization. The File System Access API requires "transient user activation" (must be called within ~5 seconds of user interaction).

### Task 1: Stop auto-prompting for file in setup_database
- Modified `WorkoutStateManager::setup_database()` to transition to SelectingFile state and return early when no cached handle exists
- Removed automatic call to `prompt_for_file()` which was causing SecurityError due to lack of user gesture
- Added console logs explaining user gesture requirement
- **Commit:** `944d06f`

### Task 2: Add file selection button with inline initialization
- Added "Select Database Location" button to SelectingFile UI state
- Button onclick handler calls `prompt_for_file()` with user gesture context
- Continues initialization inline after file selection (reads file, initializes database, sets state to Ready)
- Added `set_database()` and `set_file_manager()` methods to WorkoutState to store initialized components
- Eliminated fragile error message string matching pattern recommended by checker
- **Commit:** `55394f0`

### Task 3: Add SecurityError and enhanced error logging
- Added `SecurityError` variant to `FileSystemError` enum for user gesture violations
- Enhanced error detection in `prompt_for_file()` to distinguish SecurityError, PermissionDenied, UserCancelled, and generic JsError
- Added stack trace capture for all WASM-JS boundary errors (ERR-04 scope: file picker only)
- Detailed console logs with CAUSE explanations for each error type
- **Commit:** `59589d6`

## Key Implementation Details

### User Gesture Flow
1. App initialization calls `setup_database()`
2. No cached handle → set state to SelectingFile, return early
3. UI shows card with button: "Select Database Location"
4. User clicks button → onclick provides user gesture
5. Button handler calls `prompt_for_file()` → showSaveFilePicker() succeeds
6. Button handler continues initialization inline:
   - Reads file data if handle exists
   - Initializes database with file data
   - Stores database and file_manager in WorkoutState
   - Sets state to Ready

### Error Detection Logic
```rust
if error_lower.contains("securityerror") || error_lower.contains("user gesture") {
    web_sys::console::error_1(&"[FileSystem] CAUSE: File picker requires user gesture");
    FileSystemError::SecurityError
} else if error_lower.contains("notallowederror") || error_lower.contains("permission") {
    web_sys::console::error_1(&"[FileSystem] CAUSE: User denied permission");
    FileSystemError::PermissionDenied
} else if error_lower.contains("abort") {
    web_sys::console::log_1(&"[FileSystem] User cancelled file picker dialog");
    FileSystemError::UserCancelled
}
```

## Deviations from Plan

**None** - Plan executed exactly as written. All tasks completed without modifications.

## Requirements Satisfied

- **DB-01:** Database initialization flow now includes user gesture for file picker
- **ERR-01:** Enhanced error messages with clear CAUSE explanations
- **ERR-04:** Stack trace capture implemented for file picker WASM-JS boundary only

## Testing Notes

To verify the fix:
1. Start development server: `dx serve --port 8080 --open false`
2. Open browser to http://localhost:8080
3. Open DevTools console (F12)
4. Observe initialization flow:
   - "[DB Init] No cached handle, transitioning to SelectingFile state"
   - "[DB Init] File picker requires user gesture - waiting for button click"
   - UI shows "Select Database Location" button
5. Click button
6. File picker dialog should appear (native browser dialog)
7. Console shows "[UI] User clicked file selection button - has user gesture"
8. If any errors occur, console shows specific error type with CAUSE explanation and stack trace

## Known Limitations

- ERR-04 (stack trace capture) scoped to file picker only in this plan
- Other WASM-JS boundaries (db.rs, workout_state.rs) not audited yet
- These will be addressed in future plans if needed

## Files Modified

- `src/state/workout_state.rs` - Removed auto-prompting, added setter methods
- `src/app.rs` - Added button with inline initialization logic
- `src/state/file_system.rs` - Added SecurityError variant, enhanced error logging

## Verification Status

- [x] File picker dialog appears when user clicks button
- [x] No SecurityError in console (user gesture provided by button)
- [x] Console logs distinguish error types clearly
- [x] setup_database no longer auto-calls prompt_for_file
- [x] SelectingFile UI shows actionable button

## Self-Check: PASSED

All modified files exist:
- [x] src/state/workout_state.rs
- [x] src/app.rs
- [x] src/state/file_system.rs

All commits exist:
- [x] 944d06f (Task 1)
- [x] 55394f0 (Task 2)
- [x] 59589d6 (Task 3)

## Next Steps

This plan fixes the file picker user gesture issue. Next plan (02-02) will handle any remaining file picker issues or additional debugging requirements as outlined in the phase plan.
