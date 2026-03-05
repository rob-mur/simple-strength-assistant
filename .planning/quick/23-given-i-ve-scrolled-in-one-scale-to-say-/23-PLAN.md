---
must_haves:
  - Updates `ExerciseForm` Minimum Weight TapeMeasure to align its tick marks with the current `min_weight()` when increment changes, using modulo arithmetic for the `min` prop.
  - Adds descriptive subtext to the "Minimum Weight" label in `ExerciseForm`.
  - Adds descriptive subtext to the "Weight Increment" label in `ExerciseForm`.
---

# Quick Task 23: Fix Exercise Form Tape Measure Scaling and Explanations

## Context
When creating a new exercise in `ExerciseForm`, users can set a `Minimum Weight` and a `Weight Increment`. The tape measure for `Minimum Weight` had a fixed `min` of `0.0` and a `step` equal to the selected increment. If a user scrolled the minimum weight to a value like `12.5` (using a `2.5` increment) and then switched to a `5.0` increment, the tape measure ticks would shift to `0, 5, 10, 15` making the current `12.5` value "in between" ticks and snapping poorly. The scale should adjust to line up with the currently selected value. Additionally, the meaning of "Minimum Weight" and "Weight Increment" is confusing to some users and needs clarifying text.

## Tasks

1. **Task 1: Align Tape Measure Scale in Exercise Form**
   - **Files:** `src/components/exercise_form.rs`
   - **Action:** Update the `TapeMeasure` used for "Minimum Weight" to set `min: (min_weight() % increment()) as f64` instead of `min: 0.0`. This dynamically changes the starting phase of the tick marks, ensuring the currently selected value remains aligned on a tick mark even when the increment step changes.
   - **Verify:** Run library management tests to ensure no regressions.
   - **Done:** When the code compiles and tests pass.

2. **Task 2: Add Explanatory Text for Form Fields**
   - **Files:** `src/components/exercise_form.rs`
   - **Action:** 
     - Add subtext under "Minimum Weight": "The smallest possible weight you can do with this exercise (e.g., the weight of an empty barbell or the first pin on a machine)."
     - Add subtext under "Weight Increment": "The smallest amount of weight you can add on top of the minimum (e.g., adding two 1.25kg plates equals a 2.5kg increment)."
   - **Verify:** Code compiles and formats cleanly.
   - **Done:** When descriptions are correctly structured inside the label elements.

## PLANNING COMPLETE