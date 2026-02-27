# Pitfalls Research: Tactile Training Experience (v1.1)

## Dimension: Pitfalls

### Common mistakes when adding tactile inputs to PWA

- **Accidental Scroll**: If `touch-action: none` isn't applied, the PWA will scroll while the user tries to swipe the tape measure.
- **Pointer Capture**: Forgetting `set_pointer_capture` means if the user moves their finger too fast and leaves the element, the "up" event might be missed, leaving the component in a "stuck" dragging state.
- **Mobile Latency**: High-frequency updates of many SVG elements can be slow. Keep the "window" of visible tape measure ticks small.
- **Desktop UX**: Dragging a tape measure with a mouse can feel weird if not calibrated correctly; ensure clicking a tick mark also jumps to that value.

### Warning signs

- **Laggy UI**: If Dioxus re-renders the entire app on every pixel of movement. Use localized signals.
- **Stuck Drags**: The tape measure keeps moving after the finger is lifted.

### Prevention strategy

- Use `dioxus` 0.7 signals for localized updates.
- Use `set_pointer_capture` on `pointerdown`.
- Implement `onpointercancel` to handle system interruptions.
