---
status: diagnosed
phase: 02-debug-and-fix-file-picker
source: [.planning/phases/02-debug-and-fix-file-picker/02-01-SUMMARY.md, .planning/phases/02-debug-and-fix-file-picker/02-02-SUMMARY.md]
started: 2026-02-26T10:00:00Z
updated: 2026-02-26T10:05:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 1
name: App initialization without cached handle
expected: |
  1. Open app at http://localhost:8080 (first time or clear site data)
  2. App should NOT automatically prompt for a file.
  3. UI should show a card with "Select Database Location" button.
  4. DevTools Console should show: "[DB Init] No cached handle, transitioning to SelectingFile state" and "[DB Init] File picker requires user gesture - waiting for button click"
awaiting: user response

## Tests

### 1. App initialization without cached handle
expected: App doesn't auto-prompt; shows "Select Database Location" button; console logs explain user gesture requirement.
result: issue
reported: "the console log does say [DB Init] File picker requires user gesture - waiting for button click, but no button is actually shown, instead there's just a loading spinner saying 'initializing database'"
severity: major

### 2. File Selection via Button
expected: Clicking "Select Database Location" opens native file picker. After selection, app initializes database and enters Ready state.
result: [pending]

### 3. File Picker Cancellation
expected: Clicking button then cancelling the dialog logs "User cancelled file picker dialog" in console and stays in SelectingFile state.
result: [pending]

### 4. Initialization with Cached Handle (Permission Granted)
expected: Reload app after successful file selection. App should automatically retrieve cached handle, verify permission (granted), and enter Ready state without showing the button.
result: [pending]

### 5. Detailed Permission Logging
expected: During initialization with a cached handle, DevTools Console shows detailed logs for permission state transitions (querying, evaluating, decision points).
result: [pending]

## Summary

total: 5
passed: 0
issues: 1
pending: 4
skipped: 0

## Gaps

- truth: "UI should show a card with 'Select Database Location' button when in SelectingFile state."
  status: failed
  reason: "User reported: the console log does say [DB Init] File picker requires user gesture - waiting for button click, but no button is actually shown, instead there's just a loading spinner saying 'initializing database'"
  severity: major
  test: 1
  root_cause: "Lack of reactivity in WorkoutState (using Rc<RefCell> instead of Dioxus Signal) and setup_database returning Err for SelectingFile state transition."
  artifacts:
    - path: "src/state/workout_state.rs"
      issue: "State fields are not reactive; setup_database returns Err for normal state transition"
    - path: "src/app.rs"
      issue: "UI doesn't re-render when WorkoutState changes"
  missing:
    - "Refactor WorkoutState to use Signal<T> for reactive fields"
    - "Update setup_database to return Ok(()) for SelectingFile state"
  debug_session: .planning/debug/ui-show-button.md
