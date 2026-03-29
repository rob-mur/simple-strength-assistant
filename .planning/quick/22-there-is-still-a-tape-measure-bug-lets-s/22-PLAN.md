---
must_haves:
  - Plan updates `TapeMeasure` to use reactive signals for `step`, `min`, `max`, and `value` inside the `use_future` loop instead of static `props`.
  - Fixes bug where snapping uses old `step` value after a scale change.
  - Adds a new BDD scenario specifically testing for scale change followed by interaction to ensure the snapping uses the new scale.
---

# Quick Task 22: Fix Tape Measure Scale Snapping Bug

## Context

When the `step` scale is changed while the tape is idle, the component correctly repositions the tape because of the `last_step` signal sync block. However, the `use_future` momentum and snapping loop permanently captures the initial `props` from component mount. When the user interacts with the tape measure again (or it enters snapping), it calculates boundary clamping and snap targets using the old, statically captured `props.step`, `props.min`, `props.max`, and `props.value`.

## Tasks

1. **Task 1: Add `last_max` signal and use signals in `use_future` loop**
   - **Files:** `src/components/tape_measure.rs`
   - **Action:**
     - Add `mut last_max = use_signal(|| props.max);`
     - Update the prop-sync `if` block to also sync `last_max`.
     - In the `use_future` closure, replace `props.step`, `props.min`, `props.max`, and `props.value` with `*last_step.peek()`, `*last_min.peek()`, `*last_max.peek()`, and `*last_value.peek()`.
   - **Verify:** Run tests `cargo test`.
   - **Done:** When `TapeMeasure` compiles and snapping logic correctly calculates offsets dynamically.

2. **Task 2: Add BDD test for scale change followed by snapping**
   - **Files:**
     - `tests/features/tape_measure_core.feature`
     - `tests/steps/tape_measure_steps.rs`
   - **Action:** Add a scenario: "Scale change followed by interaction uses new scale" where tape measure starts at 10kg with step 2.5, changes to step 5.0, then is manually dragged or set to snap, and verify it snaps to a multiple of 5.0.
   - **Verify:** `./scripts/ci-test.sh` passes.
   - **Done:** When the test passes.

## PLANNING COMPLETE
