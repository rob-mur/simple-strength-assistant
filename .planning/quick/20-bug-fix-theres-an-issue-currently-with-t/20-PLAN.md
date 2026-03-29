---
must_haves:
  - Tape measure offset recalculates when `props.step` changes
  - Tape measure offset recalculates when `props.min` changes
---

# Plan: Fix TapeMeasure Scale Jump Bug

## Task 1: Update prop sync logic

- **files:** `src/components/tape_measure.rs`
- **action:** Add `props.step` and `props.min` to the prop-sync condition so that if the scale changes while dragging or otherwise, the offset is recalculated, preventing wild jumps in the displayed value.
- **verify:** Run `cargo test --test tape_measure_bdd` to ensure no regressions.
- **done:** true
