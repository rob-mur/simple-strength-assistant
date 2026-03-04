# Quick Task 22: Fix Tape Measure Scale Snapping Bug

## Execution Summary
The `TapeMeasure` component's `use_future` momentum and snapping loop was previously capturing `props.step`, `props.min`, `props.max`, and `props.value` at component initialization. This caused a bug where changing the scale (e.g., from step 2.5 to 5) would reposition the tape correctly via the reactive `if` block, but subsequent dragging or snapping would use the stale scale values for boundaries and step targets.

## Changes Made
- Added `last_max` to the prop-sync pattern in `src/components/tape_measure.rs`.
- Updated the `use_future` snapping and edge resistance logic to use the reactive `.peek()` values from the synced signals (`last_step`, `last_min`, `last_max`, and `last_value`) instead of the initial statically captured `props`.
- Added a new BDD scenario "Scale change followed by interaction uses new scale" in `tests/features/tape_measure_core.feature` to ensure snapping works correctly with the newly established scale.
- Implemented corresponding test steps in `tests/steps/tape_measure_steps.rs`.

All tests and lints pass successfully. The tape measure now behaves correctly after a step size change.