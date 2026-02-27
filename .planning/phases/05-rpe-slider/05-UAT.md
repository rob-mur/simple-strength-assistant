# UAT: RPE Slider (Phase 5)

## Objective
Verify the "no-typing" RPE slider meets the tactical and visual requirements.

## Test Scenarios

### 1. Visual Presentation
- [ ] RPE value is displayed in large, bold text.
- [ ] Value color changes based on intensity:
    - 1.0 - 5.5: Green (Success)
    - 6.0 - 7.0: Yellow/Accent (Accent)
    - 7.5 - 8.5: Orange (Warning)
    - 9.0 - 10.0: Red (Error)
- [ ] Track shows ticks for every 0.5 increment and labels for whole numbers.

### 2. Interaction & Snapping
- [ ] Dragging the slider updates the value instantly.
- [ ] Tapping on the track jumps to the nearest 0.5 increment.
- [ ] Releasing the drag maintains the snapped value.
- [ ] It is impossible to select a value outside 1.0 - 10.0.

### 3. Data Integration
- [ ] Start a workout session.
- [ ] Log a set using the RPE slider.
- [ ] Verify the "Completed Sets" table shows the exact RPE selected.
- [ ] Verify the next set prediction uses the last logged RPE as the default.

## Success Criteria
- [ ] All visual and interaction tests pass.
- [ ] No regression in set logging functionality.
