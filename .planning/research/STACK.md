# Stack Research: Tactile Training Experience (v1.1)

## Dimension: Stack

### What stack additions/changes are needed for tactile inputs?

- **Dioxus 0.7 Signals**: Use `use_signal` for high-frequency updates (e.g., current value during swipe).
- **SVG for Tape Measure**: Use SVG for rendering marking lines and numbers. This allows for scalability and simplified hit testing.
- **Pointer Events**: Use `onpointerdown`, `onpointermove`, and `onpointerup` to unifiedly handle both mouse and touch events without extra complexity.
- **CSS `touch-action: none`**: Vital on the tape measure component to prevent browser scrolling while swiping.
- **`web-sys` for Pointer Capture**: Use `set_pointer_capture` to ensure events are tracked even when the pointer leaves the component bounds during a swipe.

### Specific libraries with versions for NEW capabilities

- **`dioxus` (0.7.x)**: Current version is suitable.
- **`web-sys`**: For pointer capture API calls.

### Integration points

- **`src/app.rs`**: Integrate the new tactile components into the set recording UI.
- **`src/models/set.rs`**: Ensure models handle the precision provided by tape measure (e.g., 0.5kg increments).

### What NOT to add

- **Heavy charting libraries (yet)**: Defer until v1.2.
- **Vibration/Haptics**: User specifically requested to keep it simple and skip this for now.
