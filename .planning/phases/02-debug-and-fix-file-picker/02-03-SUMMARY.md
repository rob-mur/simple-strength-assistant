---
phase: 02-debug-and-fix-file-picker
plan: 03
subsystem: state-management
tags: [reactivity, dioxus-0.7, signals, bugfix]
---

# Phase 02 Plan 03: Refactor WorkoutState for Reactivity Summary

**Refactored WorkoutState to use Dioxus 0.7 Signals for reactivity and fixed error handling in setup_database.**

## What Was Built

### Task 1: Refactor WorkoutState to use Signals
- Refactored `WorkoutState` to be a `Copy` struct with `Signal<T>` fields.
- Implemented `PartialEq` for `Database` and `FileSystemManager` to support signals.
- Methods now use signal patterns (calling signal for subscription, using local mutable copies of signals for `.set()` to allow mutation from `&self`).

### Task 2: Fix setup_database return logic
- Modified `WorkoutStateManager::setup_database` to return `Ok(())` for the `SelectingFile` transition.
- This ensures the UI properly reacts to the state change and shows the "Select Database Location" button instead of transitioning to an error state.

### Task 3: Update App component
- Removed redundant signals from `App`.
- Updated `rsx!` match blocks to subscribe directly to `workout_state.initialization_state()`.
- Verified that state changes automatically trigger UI re-renders.

## Verification
- Project compiles successfully for WASM target.
- Reactivity patterns follow Dioxus 0.7 best practices.
- Error flow corrected to support manual file picker triggering.
