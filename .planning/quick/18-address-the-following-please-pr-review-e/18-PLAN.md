---
must_haves:
  - data-hydrated attribute is set in App component instead of WorkoutView
  - LibraryView exercise list keys use exercise id instead of name
  - WorkoutView has a comment acknowledging implicit tab mutation
  - validation.rs has a ValidationError variant for SetType mismatch or a tracking comment
  - app.rs redundant parentheses on active_tab reads are removed
---

# Quick Task 18 Plan

## 1. Relocate Hydration Attribute & Fix App.rs Parentheses

**Files:** `src/components/workout_view.rs`, `src/app.rs`
**Action:** Remove the `data-hydrated` use_effect logic from `workout_view.rs`. Add equivalent `use_effect` logic in `app.rs` to set the `data-hydrated` attribute when the application mounts. Also, fix the `(active_tab)()` calls to `active_tab()` in `app.rs`.

## 2. Update LibraryView Key and WorkoutView Comments

**Files:** `src/components/library_view.rs`, `src/components/workout_view.rs`
**Action:** In `library_view.rs`, update the `key: "{exercise.name}"` to use the exercise's ID (e.g., `key: "{exercise.id.unwrap_or(0)}"`. In `workout_view.rs`, add a comment acknowledging that `active_tab` is implicitly coupled via context, matching the existing comment in `library_view.rs`.

## 3. Add Set-Type Validation Tracking

**File:** `src/models/validation.rs`
**Action:** Find the cross-type combinations match in `src/models/validation.rs` and add a TODO/tracking comment about adding a specific `ValidationError` variant for this, as requested. (Optionally, add the variant itself if straightforward, but the PR review suggests "worth tracking as a future ValidationError variant").
