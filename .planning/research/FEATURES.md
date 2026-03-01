# Features Research: Tactile Training Experience (v1.1)

## Dimension: Features

### How do "no-typing" features typically work?

- **Swipeable Tape Measure**: A horizontal scrollable area with tick marks. Central needle indicates current value. Dragging moves the tape.
- **Big Step Buttons**: Buttons (±5, ±10, ±25) placed around the tape measure to allow quick jumps across large ranges (e.g., jumping from 40kg to 100kg).
- **Discrete Increments**: Values snap to a grid (e.g., 0.5 for kg, 1 for reps).
- **Slider for RPE**: A horizontal slider with 0.5 increments from 1 to 10. Distinct from the "infinite" tape measure as it has a fixed range.

### Table stakes vs Differentiators vs Anti-features

| Category | Feature | Note |
|----------|---------|------|
| **Table Stakes** | Swipeable Tape Measure | Essential for "no-typing" goal. |
| **Table Stakes** | Snap-to-step | Ensures clean data entry. |
| **Differentiator** | Mouse Support | Tape measure responds to clicks on desktop (simulated touch). |
| **Anti-feature** | Native Keyboard | Specifically avoiding the mobile keyboard popup. |

### Complexity noted

- Implementing smooth momentum scrolling for the tape measure in pure Dioxus/SVG might be complex; start with direct drag-to-value.
- Handling window-wide pointer events for dragging outside the component.
