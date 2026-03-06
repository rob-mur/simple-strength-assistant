# Quick Task: 24 - Format Weights Correctly

## Task Description
Change how weight numbers are formatted in the Tape Measure component:
1. For whole numbers, there should be no decimal shown (e.g. instead of 1.00 or 1.0 it would just be 1)
2. Show at most 2 decimal places (rounding in the formatting if needed)
This fixes a bug where weights with decimals (e.g., 17.5) would be rendered as whole numbers (18) if the step increment was a whole number (like 5), because the decimal precision was erroneously tied to the increment.

## Implementation Details
- Modified `src/components/tape_measure.rs` to use `format!("{}", (val * 100.0).round() / 100.0)` for rendering the major ticks. This inherently strips trailing zeros for whole numbers and supports up to 2 decimal places without forcing trailing zeros based on `props.step`.
