# Quick Plan: Make numerical displays for weight display up to 2dp

## Task Description

"please make the numerical displays for weight display up to 2dp where needed as theyre getting truncated atm."

## Tasks

1. Update `src/components/tape_measure.rs`
   - Modify the string formatting logic to use `(val * 100.0).round() / 100.0` instead of conditional `:.0` truncations to retain decimals without trailing zeros.
2. Update `src/components/exercise_form.rs`
   - Change `"{min_weight} kg"` to `"{ (min_weight() * 100.0).round() / 100.0 } kg"`
3. Update `src/components/library_view.rs`
   - Round both `min_weight` and `increment` to 2dp formatting when displaying.
4. Update `src/app.rs`
   - Apply 2dp rounding logic for `weight_input` display and `weight` display in completed sets list.
