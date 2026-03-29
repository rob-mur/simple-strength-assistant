# Quick Task 23: Fix Exercise Form Tape Measure Scaling and Explanations

## Execution Summary

The `ExerciseForm` uses a `TapeMeasure` component for the user to select the "Minimum Weight" of an exercise. The tape measure previously hardcoded its `min` property to `0.0` and its `step` property to the selected `Weight Increment`. This caused an issue where, if the user had scrolled to a specific minimum weight (e.g., 12.5) while on a small increment, and then switched to a larger increment (e.g., 5.0), the tape measure would force the tick marks to start from 0 (e.g., 0, 5, 10, 15). This left the user's selected 12.5 value visually stranded "in between" ticks, causing confusing snapping behavior when dragged.

To fix this, the `min` property of the `TapeMeasure` is now dynamically calculated using modulo arithmetic: `min: (min_weight() % increment()) as f64`. This ensures that the tape measure's ticks are always perfectly aligned with the currently selected `min_weight`, providing a smooth UX where the scale adapts to the user's selected base weight, rather than forcing the base weight to be a multiple of the increment.

Additionally, to address user confusion over the meaning of these form fields, descriptive subtext was added below both the "Minimum Weight" and "Weight Increment" labels, clarifying their purpose with concrete examples (e.g., empty barbell weight vs. weight plates).

## Changes Made

- **`src/components/exercise_form.rs`:**
  - Modified the `TapeMeasure` for "Minimum Weight" to set `min: (min_weight() % increment()) as f64`.
  - Added descriptive subtext to the "Minimum Weight" label using `.label-text-alt`.
  - Added descriptive subtext to the "Weight Increment" label using `.label-text-alt`.
- **`public/styles.css`:**
  - Rebuilt CSS to include new Tailwind utility classes used for the layout of the form labels.

All tests and linters pass successfully.
