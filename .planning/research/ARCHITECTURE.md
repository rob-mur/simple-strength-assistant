# Architecture Research: Tactile Training Experience (v1.1)

## Dimension: Architecture

### How do target features integrate with existing architecture?

- **Component-Level State**: The "transient" drag state (current offset, dragging flag) stays within the component.
- **Signal-based Sync**: On `onpointerup` or during `onpointermove`, update the parent `WorkoutState` signals.
- **SVG Rendering**: The tape measure will be a specialized SVG component that maps pixel offsets to value increments.

### Integration points

- **Set Input Row**: Replace standard `<input type="number">` with the new tactile components.
- **Event Flow**: `PointerEvent` -> `Component Offset` -> `Value Calculation` -> `Global Signal Update`.

### Data flow changes

- No major changes to `WorkoutState` or `Database` models are required. The changes are purely at the UI/Interaction layer.

### Suggested build order

1.  Generic `Slider` component (for RPE).
2.  `TapeMeasure` base component with pointer-based dragging.
3.  Integration of `TapeMeasure` into `Reps` and `Weight` inputs.
4.  Refinement: "Big Step" buttons and desktop click support.
