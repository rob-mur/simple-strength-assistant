---
must_haves:
  - Use `.chars().count()` for string length validation in `exercise_form.rs`
  - Remove `test-debug.js`, `test-screenshot.png`, `swipe-end-state.png`
  - Replace nested Option signal with `FormState` enum in `library_view.rs`
  - Centralize LocalStorage `active_tab` management in `app.rs` with a constant key
  - Use `UPSERT` (`INSERT ... ON CONFLICT ... DO UPDATE`) in `db.rs` instead of `INSERT OR REPLACE`
  - Add `aria-selected` to `tab_bar.rs` tabs
  - Use `use_memo` for `filtered_exercises` in `library_view.rs`
  - Introduce `TestSearchQuery` newtype and document HTML validation logic
---

# Plan

1. **Fix Bugs 1-3**
   - **Files:** `src/components/exercise_form.rs`, `test-debug.js`, `test-screenshot.png`, `swipe-end-state.png`, `src/components/library_view.rs`
   - **Action:** Replace `name.len()` with `name.chars().count()` in `exercise_form.rs`. Delete the three debug files. Define `enum FormState { Closed, New, Edit(ExerciseMetadata) }` in `library_view.rs` and update the state signal.
   - **Verify:** Run tests and check compiler errors.
   - **Done:** Yes

2. **Fix Bugs 4-5**
   - **Files:** `src/app.rs`, `src/components/workout_view.rs`, `src/components/library_view.rs`, `src/state/db.rs`
   - **Action:** Add `const ACTIVE_TAB_KEY: &str = "active_tab";` in `app.rs`. Move the `use_effect` writing to LocalStorage from views into `app.rs`. In `db.rs`, replace `INSERT OR REPLACE INTO exercises` with an `INSERT INTO exercises ... ON CONFLICT(name) DO UPDATE SET ...` approach.
   - **Verify:** Run tests and ensure local storage tab persistence works.
   - **Done:** Yes

3. **Accessibility, Performance, and Minor Updates**
   - **Files:** `src/components/tab_bar.rs`, `src/components/library_view.rs`, `src/models/validation.rs`, `tests/` (if needed)
   - **Action:** Add `aria-selected=true/false` in `tab_bar.rs`. Wrap `filtered_exercises` creation in `use_memo` in `library_view.rs`. Create `pub struct TestSearchQuery(pub String)` in `library_view.rs` and update test files if they inject the string. Document HTML validation in `validation.rs`.
   - **Verify:** Code compiles and lint/tests pass.
   - **Done:** Yes
