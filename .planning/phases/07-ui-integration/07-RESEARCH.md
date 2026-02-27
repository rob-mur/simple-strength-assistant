# Research 07: UI Integration & Refinement

## Goal: Tactile, Mobile-First Interaction (v1.1)

The "no-typing" interface goal requires that every interaction for recording a workout set is optimized for one-handed (thumb) use on a mobile device. This research focuses on the final integration of tactile components and the refinement of the mobile user experience.

### Key Considerations

1.  **State Synchronization Pattern**:
    - **Problem**: In Dioxus, child components using local signals for immediate feedback need to stay in sync with parent state updates (e.g., when `session.predicted` changes after a set is logged).
    - **Solution**: Use `use_effect` to synchronize local signal values with parent props whenever the parent props change. This ensures that the next set's predictions are automatically loaded into the inputs after a set is logged.

2.  **Thumb-Centric Layout**:
    - **Problem**: Standard layouts often place primary actions (like "Log Set") at the top or in the middle of the screen, which can be hard to reach on large mobile devices.
    - **Solution**: Ensure the "Log Set" button is large, prominent, and located in a comfortable "thumb zone" (usually the bottom third of the screen).

3.  **Start Session setup**:
    - **Problem**: The current "Start Session" view uses standard numeric inputs for "Starting Weight" and "Weight Increment", which triggers the keyboard and breaks the "no-typing" experience.
    - **Solution**: Replace these with the `TapeMeasure` and/or `StepControls`. This maintains consistency across the application and keeps the keyboard hidden.

4.  **Visual Feedback**:
    - **Problem**: On mobile, it's not always obvious when an action (like logging a set) has been successful without visual feedback.
    - **Solution**: Use DaisyUI's toast or alert components, or a simple color transition, to provide clear feedback after logging a set.

### Interaction Refinement

- **Tape Measure**: PIXELS_PER_STEP=60 seems comfortable for precision, but might require too much swiping for large jumps.
- **Step Controls**: The ±10 / ±5 buttons are essential for large jumps. They should be positioned close to the tape measure they control.
- **RPE Slider**: Needs large hitboxes (range-lg) to avoid precision errors on small screens.

---
*Research synthesized: 2026-02-27*
