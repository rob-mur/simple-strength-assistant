---
type: quick
task_count: 3
wave: 1
autonomous: true
files_modified:
  - src/components/tape_measure.rs
  - tests/features/tape_measure_core.feature
  - tests/features/tape_measure_physics.feature
---

<objective>
Address PR review comments for TapeMeasure component: fix external sync bug preventing button updates from scrolling the tape, replace unsafe unwraps with safe error handling, fix floating-point drift in velocity checks, add idle animation guard, and update BDD feature documentation to reflect sync behavior.

Purpose: Resolve critical bugs preventing step buttons from syncing with TapeMeasure, improve code safety and robustness, and ensure documentation matches implementation.

Output: TapeMeasure component with reliable external sync, safe error handling, and accurate BDD documentation.
</objective>

<context>
## Background
Phase 6 UAT revealed that step buttons update the displayed value but fail to scroll the TapeMeasure to the new position. Root cause analysis identified:

1. **Sync bug (lines 33-38)**: External sync condition `!*is_dragging.peek() && !*is_snapping.peek()` is too restrictive - if residual velocity exists (below VELOCITY_THRESHOLD but non-zero), sync never triggers
2. **Unsafe unwraps (lines 132, 145, 186, 199)**: `.unwrap()` calls on `downcast::<PointerEvent>()` will panic if event type changes
3. **Float drift**: Velocity comparison `current_velocity_val.abs() > VELOCITY_THRESHOLD` may fail due to floating-point precision issues
4. **No idle guard**: Animation loop runs continuously even when component is idle (not dragging, not moving, not snapping)
5. **BDD docs outdated**: Feature files don't document external sync behavior from prop changes

## Current Implementation Issues

From `.planning/phases/06-jump-controls/06-UAT.md`:
- Test 6 FAILED: "tapping does update the displayed number but it doesmt update the tape measure"
- Diagnosis: "The `use_effect` in `TapeMeasure` responsible for external sync has a condition `*velocity.peek() == 0.0`. If a previous interaction left a tiny residual velocity (below `VELOCITY_THRESHOLD`), the sync never triggers."

From code review:
- Lines 33-38: Sync only fires when `!is_dragging && !is_snapping`, but doesn't reset velocity
- Lines 50, 79: Velocity checks use direct comparison which is fragile with floating-point math
- Lines 132, 145, 186, 199: Unsafe `.unwrap()` calls on event downcasts
- Lines 41-101: Animation loop has no early exit when component is idle

@src/components/tape_measure.rs
@.planning/phases/06-jump-controls/06-UAT.md
</context>

<tasks>

<task type="auto">
  <name>Fix TapeMeasure sync bug and add robust float handling</name>
  <files>src/components/tape_measure.rs</files>
  <action>
**Fix external sync (lines 33-38):**
- When props.value changes and component is not actively dragging or snapping, immediately reset velocity to 0.0 and force offset update
- Change condition from checking velocity separately to resetting it as part of sync
- Ensure sync happens BEFORE animation loop reads state

**Fix floating-point drift in velocity checks (lines 50, 75, 79, 189):**
- Replace `current_velocity_val.abs() > VELOCITY_THRESHOLD` with `current_velocity_val.abs() >= VELOCITY_THRESHOLD + f64::EPSILON` to handle float precision
- Replace `new_v.abs() <= VELOCITY_THRESHOLD` with `new_v.abs() < VELOCITY_THRESHOLD` for clearer logic
- Replace `*velocity.peek() == 0.0` with `velocity.peek().abs() < f64::EPSILON`
- Use consistent epsilon-based comparisons for all floating-point checks

**Add idle animation guard (lines 45-48):**
- Skip animation loop iteration if ALL conditions true: not dragging, velocity is effectively zero (< EPSILON), and not snapping
- Add early `continue` at top of loop when component is idle
- This prevents unnecessary CPU usage when component is static

**WHY these specific fixes:**
- Sync bug: Previous check allowed residual velocity to block sync. Now we force velocity to 0.0 during sync, guaranteeing offset update triggers.
- Float drift: Direct float equality/comparison is unreliable. Using EPSILON ensures comparisons work despite floating-point representation errors.
- Idle guard: Without it, loop runs 60 times/second even when nothing is moving, wasting battery on mobile devices.
  </action>
  <verify>
    <automated>cargo check</automated>
  </verify>
  <done>
    - External prop changes immediately update TapeMeasure position
    - Velocity comparisons use epsilon-based checks
    - Animation loop skips iterations when component is idle
    - No compiler warnings about float comparison
  </done>
</task>

<task type="auto">
  <name>Replace unsafe unwraps with safe error handling</name>
  <files>src/components/tape_measure.rs</files>
  <action>
**Replace all `.unwrap()` calls on event downcasts (lines 132, 145, 186, 199):**
- Replace `evt.data.downcast::<PointerEvent>().unwrap()` with `if let Some(e) = evt.data.downcast::<PointerEvent>() { ... } else { return; }`
- For `onpointerdown`, `onpointermove`, `onpointerup`, `onpointercancel`: gracefully return early if downcast fails
- Add debug log for unexpected event types: `web_sys::console::log_1(&"Unexpected event type in TapeMeasure handler".into());` (only in else branch)

**WHY safe handling instead of unwrap:**
- `.unwrap()` will panic the entire WASM module if event type ever changes (browser update, Dioxus update, etc.)
- Early return is safe: if we get a non-PointerEvent, it means the event binding is wrong or browser sent unexpected event - ignoring it is safer than crashing
- Debug log helps identify issues during development without breaking production

**DO NOT** change pointer capture calls - those already use `let _ = el.set_pointer_capture()` pattern which safely ignores errors.
  </action>
  <verify>
    <automated>cargo check && cargo clippy -- -D warnings</automated>
  </verify>
  <done>
    - No `.unwrap()` calls on event downcasts
    - All pointer event handlers use safe `if let Some()` pattern
    - Clippy warnings cleared
    - Code cannot panic from unexpected event types
  </done>
</task>

<task type="auto">
  <name>Update BDD feature files to document external sync behavior</name>
  <files>tests/features/tape_measure_core.feature, tests/features/tape_measure_physics.feature</files>
  <action>
**Add to `tape_measure_core.feature` (new scenario):**
```gherkin
Scenario: External value changes update tape position
  Given the TapeMeasure is initialized with value 100kg
  When the parent component updates the value prop to 150kg
  Then the tape should immediately scroll to center 150kg
  And the offset should reflect the new position
  And velocity should be reset to 0.0
```

**Add to `tape_measure_physics.feature` (new scenario):**
```gherkin
Scenario: External updates during idle state
  Given the TapeMeasure is idle (not dragging, no momentum)
  When the value prop changes from external source
  Then the animation loop should immediately sync the offset
  And no snapping animation should play
  And the new value should be centered instantly
```

**Update existing scenarios if needed:**
- Review scenarios for any references to prop changes
- Ensure no conflicts with new sync behavior
- Add "idle animation guard" scenario if performance scenarios exist

**WHY update BDD docs:**
- Current feature files don't document external sync from prop changes
- Phase 04 focused on user dragging/physics, not programmatic updates
- These scenarios prevent regression of the sync bug we're fixing
  </action>
  <verify>
    <automated>grep -q "External value changes update tape position" tests/features/tape_measure_core.feature && grep -q "External updates during idle state" tests/features/tape_measure_physics.feature</automated>
  </verify>
  <done>
    - BDD feature files include scenarios for external value sync
    - Scenarios document idle state behavior
    - Feature files accurately reflect TapeMeasure sync capabilities
  </done>
</task>

</tasks>

<verification>
**Manual testing:**
1. Start active session
2. Use step buttons (±10 for weight, ±1 for reps)
3. Verify TapeMeasure immediately scrolls to center the new value
4. Drag TapeMeasure manually, verify physics still work
5. Use step buttons again, verify they override dragged position
6. Open browser DevTools Performance tab, verify no continuous animation when idle

**Expected behavior:**
- Step buttons instantly update both displayed number AND tape position
- No delay or failed sync after button press
- No console errors from event handling
- CPU usage drops to ~0% when TapeMeasure is idle (not moving)
</verification>

<success_criteria>
- [ ] Step buttons update TapeMeasure position immediately (UAT Test 6 regression)
- [ ] No `.unwrap()` calls on event downcasts (safe error handling)
- [ ] Velocity comparisons use epsilon-based checks (no float drift issues)
- [ ] Animation loop exits early when component is idle (battery efficient)
- [ ] BDD feature files document external sync behavior
- [ ] `cargo check` and `cargo clippy` pass with no warnings
- [ ] Manual testing confirms buttons sync with tape measure
</success_criteria>

<output>
After completion, update `.planning/STATE.md` to reflect quick fix completed and component ready for final review.
</output>
