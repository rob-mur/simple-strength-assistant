# Research: RPE Slider (Phase 5)

## Objective
Implement a discrete, tactile slider for recording RPE (Rate of Perceived Exertion) in 0.5 increments from 1.0 to 10.0.

## UI/UX Goals
1. **No-typing interface**: User should only need to drag or tap to select RPE.
2. **Tactile feedback**: Clear snapping to increments.
3. **Visual clarity**: Selected value must be prominent.
4. **Mobile-first**: Large touch targets for thumb interaction.

## Technical Options

### Option 1: Native `<input type="range">`
- **Pros**:
  - Built-in accessibility and snapping.
  - Minimal implementation effort.
  - Native performance.
- **Cons**:
  - Hard to style consistently across browsers (especially for "thick" or "fancy" sliders).
  - Native "snap" can feel mushy without custom physics.

### Option 2: Custom SVG Slider (like TapeMeasure)
- **Pros**:
  - Total control over aesthetics.
  - Smooth snapping and momentum if needed.
  - Consistent across browsers.
- **Cons**:
  - More complex implementation (pointer events, math).
  - Accessibility requires manual work (`aria-valuenow`, etc.).

### Option 3: Discrete Button Segmented Control
- **Pros**:
  - Extremely tactile (clear boundaries).
  - No dragging required, just taps.
- **Cons**:
  - 19 buttons (1.0 to 10.0 in 0.5 steps) might be too crowded for a single row.
  - Doesn't feel like a "slider".

## Recommendation
Use a **Custom Styled Range Input** or a **Custom Dioxus Component with Pointer Events** that mimics a slider but with larger vertical hitboxes. Given the "tactile" requirement and the success of `TapeMeasure`, a custom component using pointer events is preferred for consistency and "premium" feel.

## Snap Logic
- Range: 1.0 - 10.0
- Step: 0.5
- Snap: Round to nearest 0.5 on release or during drag for immediate feedback.

## Visual Feedback
- Labels for whole numbers (1, 2, 3, ... 10).
- Subtle ticks for half steps.
- Color coding:
  - 9.0 - 10.0: Red (High intensity)
  - 7.5 - 8.5: Orange (Moderate/High)
  - 6.0 - 7.0: Yellow (Moderate)
  - < 6.0: Green (Low intensity/Warm-up)

## Integration Plan
1. Create `src/components/rpe_slider.rs`.
2. Add `RPESlider` to `src/components/mod.rs`.
3. Replace the `input[type="number"]` in `src/app.rs` with `RPESlider`.
4. Update `ActiveSession` state to use `f64` for RPE consistency with `TapeMeasure`.
