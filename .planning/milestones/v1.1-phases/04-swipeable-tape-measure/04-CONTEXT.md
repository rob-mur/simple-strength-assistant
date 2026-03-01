# Phase 04 Context: Swipeable Tape Measure

## 1. Visual Indicators & Feedback
- **Aesthetic:** Opacity-driven focus. The central (active) value has full opacity, fading out as values move further left/right. 
- **Ticks:** No traditional line-heavy tick marks. Instead, a small tick mark under each number, with a slightly larger central tick for the "focus" point.
- **Positioning:** The central focus point is fixed; numbers/ticks slide behind it.
- **Numerical Display:** Integrated directly into the SVG component.
- **Animation:** Continuous animation during swipes/movements to ensure clear visual feedback.

## 2. Swipe Physics & Sensitivity
- **Momentum:** Acceleration-based. Snappy for short movements, but the duration and speed of the "glide" increase with the force/number of swipes (similar to a roulette wheel).
- **Sensitivity:** The viewport must display at least the current value and one increment above/below (`[value-step, value, value+step]`).
- **Snapping:** Must snap cleanly to the nearest increment after motion stops. A "tap" during motion should immediately stop the wheel and snap to the current value.
- **Edge Behavior:** "Hard wall" resistance when hitting minimum/maximum values.

## 3. Value Constraints & Edge Behavior
- **Reusability:** The component is generic. Minimum, maximum, and starting values are passed as inputs.
- **Minimum Value:** The absolute floor is the first increment (e.g., if step is 0.5, the minimum is 0.5).
- **Initial State:** On first load, it should center on the "previous" value (provided as an input).
- **Increments:** Configurable (e.g., 0.5 for weight, 1 for reps).

## 4. Desktop & Mouse Ergonomics
- **Interaction:** Clicks on specific numbers or areas of the tape trigger an immediate animated "jump" to center that value.
- **Excluded Features:** No support for scroll wheels, keyboard arrows, or hover effects is required for this phase.

## 5. Testing Strategy (BDD)
- **Framework:** Behavioral Driven Development (BDD) using Playwright for UI/E2E and standard Rust test suites for state/calculations.
- **Verification Flow:** 
  1. Create a "playground" environment to manually tune "feel" variables (acceleration, friction, snapping).
  2. Codify the "good feel" into Playwright tests to ensure regression-free behavior.
- **Critical Test Case:** "Roulette" momentum + immediate tap-to-stop must result in a clean snap to the value under the needle.
- **Target:** Chrome (mobile dimensions) only.

## Deferred Ideas
- **Visual Regression:** Defer specific visual snapshot testing for the opacity gradients until the core logic is stable.
- **Advanced Haptics:** Not in scope for v1.1.
- **Multiple Browsers:** Safari/Firefox support deferred.
