# Summary: Fix TapeMeasure Scale Jump Bug

## What was done
- Identified that the prop-sync pattern in `TapeMeasure` only monitored `props.value` for changes.
- Updated `src/components/tape_measure.rs` to also monitor `props.step` and `props.min` via `use_signal`.
- Recalculating the `offset` correctly when `step` or `min` changes prevents wild jumps or sticking issues when the user changes scale during interactions.

## Verification
- TapeMeasure tests pass successfully.
- Code reviewed for logic errors.