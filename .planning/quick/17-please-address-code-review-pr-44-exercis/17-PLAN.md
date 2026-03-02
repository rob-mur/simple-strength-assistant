# Quick Task 17: PR #44 Code Review Fixes

## Goal
Address code review comments for PR #44 (STR-31), improving correctness, design, performance, and testing coverage.

## Tasks

### Task 1: Bug Fixes and Correctness
- **Files**: `src/components/tape_measure.rs`, `src/state/db.rs`, `src/app.rs`, `src/components/library_view.rs`
- **Actions**:
  - `tape_measure.rs`: Remove duplicate `el.set_pointer_capture` and replace `web_sys::console::log_1` with `log::debug!`. Run `cargo fmt` on this file.
  - `db.rs`: Add `ExerciseNotFound` error variant when `UPDATE ... WHERE id = ?` returns 0 rows. Surface a meaningful message.
  - `app.rs`: Remove redundant `WorkoutStateManager::sync_exercises` spawn. Make `ACTIVE_TAB_KEY` `pub(crate)`.
  - `library_view.rs`: Add a comment making the ordering dependency explicit for `on_save` (`show_form.set(FormState::Closed)` only executes after `sync_exercises`).

### Task 2: Code Quality, Performance, and Design
- **Files**: `src/components/library_view.rs`, `src/components/exercise_form.rs`, `src/components/workout_view.rs`, `src/state/workout_state.rs`, `devenv.nix`
- **Actions**:
  - `library_view.rs`: Document implicit context coupling for `active_tab` (and `workout_view.rs`).
  - `library_view.rs`: Remove unnecessary double clone in edit button closure.
  - `library_view.rs`: Hoist `search_query().to_lowercase()` above `.filter`.
  - `exercise_form.rs`: Read `initial_exercise` directly during signal initialization. Add comment explaining default `increment` 2.5. Add comment about XSS validation.
  - `workout_state.rs`: Refactor or document double clone in `exercises()`.
  - `devenv.nix`: Remove `GOOGLE_CLOUD_PROJECT` artefact.

### Task 3: Testing and Miscellaneous
- **Files**: `tests/steps/`, `src/state/db_tests.rs`, `tests/features/exercise_list.feature`, `tests/features/exercise_search.feature`
- **Actions**:
  - BDD tests: Ensure `todo!()` steps are guarded with `#[ignore]` or removed so CI doesn't fail.
  - `db_tests.rs`: Add test for `UPDATE ... WHERE id = ?` path.
  - End-to-End Test Files: Add missing trailing newlines.
