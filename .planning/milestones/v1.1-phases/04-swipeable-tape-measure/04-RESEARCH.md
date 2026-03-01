# Phase 04: Swipeable Tape Measure - Research

**Researched:** 2026-02-27
**Domain:** SVG, Pointer Events, Momentum Physics, Dioxus 0.7.2
**Confidence:** HIGH

## Summary

The Tape Measure component is a specialized slider that provides a tactile, "no-typing" interface for inputting weight and reps. It leverages SVG for high-performance rendering and Pointer Events for cross-platform (mobile/desktop) interaction. The core challenge is achieving a "good feel" for the momentum scrolling and snapping, which will be addressed using a dedicated animation loop and configurable physics constants.

**Primary recommendation:** Build a custom SVG component using Dioxus signals to track offset and velocity, utilizing `set_pointer_capture` for reliable dragging and a `use_future` loop for momentum decay.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Aesthetic:** Opacity-driven focus. The central (active) value has full opacity, fading out as values move further left/right. 
- **Ticks:** No traditional line-heavy tick marks. Instead, a small tick mark under each number, with a slightly larger central tick for the "focus" point.
- **Positioning:** The central focus point is fixed; numbers/ticks slide behind it.
- **Numerical Display:** Integrated directly into the SVG component.
- **Animation:** Continuous animation during swipes/movements to ensure clear visual feedback.
- **Momentum:** Acceleration-based. Snappy for short movements, but the duration and speed of the "glide" increase with the force/number of swipes (similar to a roulette wheel).
- **Sensitivity:** The viewport must display at least the current value and one increment above/below (`[value-step, value, value+step]`).
- **Snapping:** Must snap cleanly to the nearest increment after motion stops. A "tap" during motion should immediately stop the wheel and snap to the current value.
- **Edge Behavior:** "Hard wall" resistance when hitting minimum/maximum values.
- **Reusability:** The component is generic. Minimum, maximum, and starting values are passed as inputs.
- **Minimum Value:** The absolute floor is the first increment (e.g., if step is 0.5, the minimum is 0.5).
- **Initial State:** On first load, it should center on the "previous" value (provided as an input).
- **Increments:** Configurable (e.g., 0.5 for weight, 1 for reps).
- **Interaction:** Clicks on specific numbers or areas of the tape trigger an immediate animated "jump" to center that value.

### Claude's Discretion
- **Physics Constants:** Tuning of acceleration, friction, and snapping speed.
- **SVG Viewbox Dimensions:** Recommended 300x80 or 400x100 for optimal thumb spacing.
- **Implementation of Snapping:** Spring-based vs. Ease-out interpolation.

### Deferred Ideas (OUT OF SCOPE)
- **Visual Regression:** Defer specific visual snapshot testing for the opacity gradients until the core logic is stable.
- **Advanced Haptics:** Not in scope for v1.1.
- **Multiple Browsers:** Safari/Firefox support deferred.
- **Excluded Desktop Features:** No support for scroll wheels, keyboard arrows, or hover effects.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TAPE-01 | Swipe to adjust weight | Pointer Events + Offset calculation |
| TAPE-02 | Swipe to adjust reps | Same as TAPE-01 with step=1 |
| TAPE-03 | Snap to increments (0.5/1.0) | Math.round(offset/step) logic in pointerup |
| TAPE-04 | Desktop support (click to jump) | `onclick` on SVG elements to set target offset |
| TAPE-05 | Scroll locking (`touch-action: none`) | Standard CSS property on container |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Dioxus | 0.7.2 | UI Framework | Project Standard |
| web-sys | 0.3 | Browser API access | Required for `set_pointer_capture` |
| wasm-bindgen | 0.2 | JS/Rust interop | Core dependency |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| instant | 0.1 | High-resolution timing | Calculating velocity between frames |

## Architecture Patterns

### Recommended Component Structure
```
src/
└── components/
    └── tape_measure/
        ├── mod.rs        # Component definition
        ├── physics.rs    # Momentum and snapping logic
        └── view.rs       # SVG rendering
```

### Momentum Pattern: Exponential Decay
**What:** Multiply velocity by a friction constant (e.g., 0.95) every frame.
**When to use:** During the glide phase after pointer release.
**Implementation:**
```rust
// In a use_future loop
if !is_dragging {
    velocity *= friction;
    offset += velocity;
    if velocity.abs() < threshold {
        start_snapping();
    }
}
```

### View Pattern: Dynamic Opacity
**What:** Calculate opacity based on distance from the viewport center.
**When to use:** Rendering each number/tick group.
**Formula:** `opacity = (1.0 - (x - center).abs() / max_distance).max(0.0).powf(2.0)`

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SVG Layout | Absolute Positioning | `viewBox` + `transform` | Handles scaling and resolution natively |
| Pointer Capture | Custom state tracking | `set_pointer_capture` | Ensures events are received even if pointer leaves element |

## Common Pitfalls

### Pitfall 1: Ghost Clicks
**What goes wrong:** A swipe is interpreted as a "click" or "tap", triggering an unintended jump.
**How to avoid:** Use a distance threshold (e.g., 5px). If the pointer moved more than the threshold, it's a drag, not a click.

### Pitfall 2: Performance Jitter
**What goes wrong:** Re-rendering many SVG elements at 60fps causes lag.
**How to avoid:** Optimize the number of rendered elements. Only render elements within the viewport + a small buffer. Use `dioxus` 0.7 signals for fine-grained updates.

## Code Examples

### Pointer Capture in Dioxus 0.7.2
```rust
use dioxus::prelude::*;
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;

#[component]
fn TapeMeasure() -> Element {
    let mut is_dragging = use_signal(|| false);
    
    rsx! {
        svg {
            onpointerdown: move |e| {
                if let Some(target) = e.raw_event().target() {
                    let el: HtmlElement = target.unchecked_into();
                    let _ = el.set_pointer_capture(e.pointer_id());
                }
                is_dragging.set(true);
            },
            onpointerup: move |_| is_dragging.set(false),
            onpointermove: move |e| {
                if is_dragging() {
                    // Update offset...
                }
            },
            // SVG Content...
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Mouse Events | Pointer Events | Standardized | Unified touch/mouse handling |
| Manual RAF | `use_future` | Dioxus 0.5+ | Easier async loop management |

## Open Questions

1. **Pixel Density vs. "Feel":** How many pixels should correspond to one increment (e.g., 0.5kg)?
   - **Recommendation:** Start with 60px per major unit (e.g., 1kg) and adjust in the "playground" mentioned in CONTEXT.md.
2. **Spring Snapping:** Should we use a simple linear interpolation or a physical spring for snapping?
   - **Recommendation:** Start with a simple Ease-out for MVP, upgrade to spring if it feels "dead".

## Sources

### Primary (HIGH confidence)
- Dioxus 0.7.2 Docs - Signal-based reactivity and Event system.
- web-sys Docs - Pointer Capture API.

### Secondary (MEDIUM confidence)
- MDN Pointer Events - Implementation details for `touch-action`.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Core project libraries.
- Architecture: HIGH - Proven momentum scrolling patterns.
- Pitfalls: MEDIUM - UX feel is subjective and requires tuning.

**Research date:** 2026-02-27
**Valid until:** 2026-03-27
