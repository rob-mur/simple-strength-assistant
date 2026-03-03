# Phase 7: Minimum Weight Implementation - Research

## Overview
This phase replaces the "Starting Weight" concept with "Minimum Weight" across the application, and updates the workout session suggestion engine to use the most recent recorded weight, falling back to the minimum weight.

## Current State Analysis
1. **Model:** `ExerciseMetadata` currently stores `min_weight: f32` inside `SetTypeConfig::Weighted`. The model itself is mostly correct.
2. **Defaults:** The default value for `min_weight` is currently hardcoded to `45.0` in both `src/components/exercise_form.rs` and `src/state/workout_state.rs` (in tests/mocks).
3. **UI (CONF-03):** `src/components/exercise_form.rs` displays "Starting Weight (kg)". It needs to be renamed to "Minimum Weight (kg)".
4. **Suggestions (SUGG-01 & SUGG-02):** 
   - When a new session is started via `WorkoutStateManager::start_session`, it calls `calculate_initial_predictions`, which currently hardcodes the prediction to `min_weight`.
   - There is currently no database function to fetch the last session or last set for an exercise. `src/state/db.rs` only has `get_exercises()`.

## Required Changes

### 1. Database Layer
- Add a new method to `Database` in `src/state/db.rs` to fetch the last recorded set for a specific exercise ID.
  - E.g., `pub async fn get_last_set_for_exercise(&self, exercise_id: i64) -> Result<Option<CompletedSet>, DatabaseError>`
  - SQL query needs to join sessions and sets to find the most recent set by timestamp/id for the given exercise.

### 2. State & Suggestion Engine
- Update `WorkoutStateManager::start_session` to be `async` and await the `get_last_set_for_exercise` query before calculating predictions.
- Update `calculate_initial_predictions` (or create a new async suggestion flow) to:
  1. Try to use the weight from the last set.
  2. If no last set exists, fallback to `min_weight` (SUGG-02).

### 3. UI Updates
- In `src/components/exercise_form.rs`:
  - Change default `min_weight` from `45.0` to `0.0`.
  - Change label from "Starting Weight (kg)" to "Minimum Weight (kg)".

### 4. Tests
- Update existing BDD tests and Playwright E2E tests that assume "Starting Weight" or a default of 45.0. 
- `tests/steps/workout_steps.rs` and `tests/steps/library_steps.rs` likely have hardcoded "45.0" and "Starting Weight" text checks.

## Validation Architecture
- **Unit Tests:** Add tests to `src/state/workout_state.rs` to verify that predictions correctly use previous session data when available.
- **DB Tests:** Add tests in `src/state/db_tests.rs` for the new `get_last_set_for_exercise` method.
- **E2E/BDD:** Ensure the UI labels are checked properly and that creating a new exercise defaults the minimum weight to 0.

## RESEARCH COMPLETE