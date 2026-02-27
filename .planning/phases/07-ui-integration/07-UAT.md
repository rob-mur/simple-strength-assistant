# UAT 07: UI Integration & Refinement

## Goal: Full integration of tactile components and mobile-first UX polish.

### 1. Start Session Setup (no-typing)
- [ ] On the "Start Session" view, the "Starting Weight" input is a swipable `TapeMeasure`.
- [ ] On the "Start Session" view, the "Weight Increment" input uses tactile controls (e.g., a picker or a small `TapeMeasure`) instead of a standard numeric input.
- [ ] The keyboard never appears when starting a session.

### 2. Active Session Integration
- [ ] The "Weight" input uses `TapeMeasure` and `StepControls`.
- [ ] The "Reps" input uses `TapeMeasure` and `StepControls`.
- [ ] The "RPE" input uses `RPESlider`.
- [ ] The "Log Set" button is large and easy to reach with a thumb.
- [ ] After logging a set, the inputs for "Weight", "Reps", and "RPE" automatically update to the *next* set's predictions.

### 3. Mobile Responsiveness
- [ ] All components fit on a 375px wide screen without horizontal scrolling.
- [ ] Touch targets are large enough (min 44x44px where possible).
- [ ] `touch-action: none` prevents accidental page scrolling while swiping components.

### 4. End-to-end Flow
- [ ] Start a session for "Squat" with 100kg starting weight and 5kg increment.
- [ ] Log 3 sets (e.g., 100kg x 8 @ 7, 100kg x 8 @ 7.5, 100kg x 8 @ 8).
- [ ] Verify that weight/reps/rpe were logged correctly after each set.
- [ ] Verify that predictions for the next set were updated after each log.
- [ ] Complete the session and verify it is correctly saved to the database.
