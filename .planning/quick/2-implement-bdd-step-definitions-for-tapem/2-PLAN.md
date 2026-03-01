---
phase: quick-2
plan: 2
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - tests/tape_measure_bdd.rs
  - tests/steps/mod.rs
  - tests/steps/tape_measure_steps.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "BDD tests can run via `cargo test --test tape_measure_bdd`"
    - "Feature files parse and execute with step definitions"
    - "Core interaction scenarios are verified (dragging, snapping, momentum)"
  artifacts:
    - path: "Cargo.toml"
      provides: "Cucumber-rs dependency for BDD testing"
      contains: "cucumber"
    - path: "tests/tape_measure_bdd.rs"
      provides: "BDD test runner and world setup"
      min_lines: 30
    - path: "tests/steps/tape_measure_steps.rs"
      provides: "Step definitions matching feature file scenarios"
      min_lines: 100
  key_links:
    - from: "tests/tape_measure_bdd.rs"
      to: "tests/features/*.feature"
      via: "Cucumber test discovery"
      pattern: "cucumber::World"
    - from: "tests/steps/tape_measure_steps.rs"
      to: "src/components/tape_measure.rs"
      via: "Component behavior verification"
      pattern: "TapeMeasure"
---

<objective>
Implement executable BDD step definitions for TapeMeasure feature files to verify core interaction behaviors.

Purpose: Enable automated testing of the TapeMeasure component's dragging, momentum, snapping, and synchronization behaviors as documented in existing .feature files.

Output: Working BDD test suite using cucumber-rs that executes all scenarios in tape_measure_core.feature and tape_measure_physics.feature.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@tests/features/tape_measure_core.feature
@tests/features/tape_measure_physics.feature
@src/components/tape_measure.rs

## Project Context

This is a Dioxus 0.7.2 WASM project. The TapeMeasure component is a tactile SVG-based input control with:
- Pointer event handling (drag, momentum, snapping)
- Physics-based animations (friction, velocity)
- External value synchronization
- Edge clamping

Feature files already exist documenting expected behaviors. This task creates the test infrastructure to verify them.

## Testing Approach

Since this is a WASM component with browser APIs (PointerEvent, DOM), we'll use:
- **cucumber-rs** for BDD framework
- **Headless simulation** of component state (not full browser integration)
- **Unit-level verification** of physics calculations and state transitions

Full end-to-end browser testing would require additional tooling (wasm-pack, headless browser). This plan focuses on verifiable state-based testing.
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add cucumber-rs dependency and create BDD test runner</name>
  <files>
    Cargo.toml
    tests/tape_measure_bdd.rs
  </files>
  <action>
Add cucumber-rs to dev-dependencies in Cargo.toml:

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
cucumber = { version = "0.21", features = ["macros"] }
tokio = { version = "1", features = ["rt", "macros"] }
```

Create tests/tape_measure_bdd.rs as the test runner:
- Define a `TapeMeasureWorld` struct implementing `cucumber::World` to hold component state
- Include test state fields: offset, velocity, is_dragging, is_snapping, value, min, max, step
- Add helper methods for simulating pointer events and physics updates
- Set up the Cucumber builder to discover .feature files in tests/features/
- Use #[tokio::main] for async test execution
- Reference step definitions module

This file is the entry point that cucumber runs. It must:
1. Implement World trait with #[derive(World, Debug, Default)]
2. Include the steps module: mod steps;
3. Call Cucumber::new().run_and_exit("tests/features").await

Why not full browser integration: WASM components require browser APIs that are hard to mock. We simulate component state transitions instead of rendering actual DOM.
  </action>
  <verify>
    <automated>cargo test --test tape_measure_bdd --no-run</automated>
  </verify>
  <done>
Cargo builds successfully with cucumber-rs dependency. Test runner file exists and compiles without errors.
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement step definitions for core interaction scenarios</name>
  <files>
    tests/steps/mod.rs
    tests/steps/tape_measure_steps.rs
  </files>
  <action>
Create tests/steps/mod.rs:
```rust
pub mod tape_measure_steps;
```

Create tests/steps/tape_measure_steps.rs with step definitions using #[given], #[when], #[then] macros.

Implement steps for tape_measure_core.feature:

**Smooth dragging scenario:**
- Given: Initialize TapeMeasureWorld with default values (value=100, min=0, max=300, step=2.5)
- When "I press down on the component at X 100": Set is_dragging=true, record pointer position
- When "I move the pointer to X 150": Calculate delta (50), update offset += 50
- Then "offset should increase by 50 units": Assert offset delta matches expected

**Scroll locking scenario:**
- Given: Component initialized
- Then "should have touch-action: none": This is UI-level (document in step as verified by manual inspection)
- Then "browser default scrolling prevented": Document as verified via pointer capture

**Pointer capture scenario:**
- Given: Component initialized
- When "I press down": Set is_dragging=true, simulate setPointerCapture
- When "I move outside boundaries": Continue processing moves
- Then "still receive pointer move events": Assert moves are processed regardless of position

**External value changes:**
- Given: Initialize with value=100kg
- When "parent updates value prop to 150kg": Simulate prop change, update world.value
- Then "tape should scroll to center 150kg": Calculate expected offset: (150-min)/step * -60.0
- Then "offset should reflect new position": Assert offset matches calculation
- Then "velocity should be reset to 0.0": Assert velocity == 0.0

Use pattern matching on Gherkin step text to extract numeric values (regex captures for kg, units, X positions).

For each step, update the TapeMeasureWorld state to match component behavior from src/components/tape_measure.rs.
  </action>
  <verify>
    <automated>cargo test --test tape_measure_bdd -- --name "Smooth dragging"</automated>
  </verify>
  <done>
Core interaction scenarios (4 scenarios in tape_measure_core.feature) pass when executed. Step definitions correctly simulate component state transitions.
  </done>
</task>

<task type="auto">
  <name>Task 3: Implement step definitions for physics scenarios</name>
  <files>
    tests/steps/tape_measure_steps.rs
  </files>
  <action>
Extend tests/steps/tape_measure_steps.rs with physics scenario steps for tape_measure_physics.feature.

**Momentum glide scenario:**
- Given "component is dragging": Set is_dragging=true
- When "I release at velocity 100 units/frame": Set is_dragging=false, velocity=100.0
- Then "should continue to glide": Simulate physics loop iteration, apply FRICTION (0.85)
- Then "offset should continue to change": Assert offset changes after release

**Snapping scenario:**
- Given "component is gliding": Set is_dragging=false, velocity=20.0
- When "velocity falls below threshold": Simulate iterations until velocity < 0.5
- Then "should interpolate toward nearest step": Set is_snapping=true, calculate target_offset
- Then "final offset exact multiple of step width": Assert (offset / 60.0) % 1.0 ≈ 0.0

**Edge resistance scenario:**
- Given "at minimum value": Set offset to max_offset (0.0 for min value)
- When "try to drag past boundary": Attempt to set offset > 0.0
- Then "offset should be clamped": Assert offset == 0.0
- Then "value should not go below minimum": Assert calculated value >= min

**Tap to stop scenario:**
- Given "component is gliding": Set is_dragging=false, velocity=50.0
- When "I press down": Set is_dragging=true, velocity=0.0, is_snapping=true
- Then "glide should stop": Assert velocity == 0.0
- Then "snap to nearest increment": Verify is_snapping==true

**External updates during idle scenario:**
- Given "component is idle": Set is_dragging=false, velocity=0.0, is_snapping=false
- When "value prop changes from external source": Update world.value from 100 to 150
- Then "animation loop should sync offset": Calculate and set new offset immediately
- Then "no snapping animation": Assert is_snapping remains false
- Then "new value centered instantly": Assert offset matches (150-min)/step * -60.0

Constants from src/components/tape_measure.rs:
- PIXELS_PER_STEP = 60.0
- FRICTION = 0.85
- VELOCITY_THRESHOLD = 0.5
- SNAP_STIFFNESS = 0.25

Use these in physics calculations to match component behavior.
  </action>
  <verify>
    <automated>cargo test --test tape_measure_bdd</automated>
  </verify>
  <done>
All physics scenarios (5 scenarios in tape_measure_physics.feature) pass. Complete test suite executes successfully with 9 total scenarios passing.
  </done>
</task>

</tasks>

<verification>
Run full BDD test suite:
```bash
cargo test --test tape_measure_bdd
```

Expected output:
- 9 scenarios executed (4 core + 5 physics)
- All steps defined and passing
- No pending or undefined steps
- Test execution completes in <5 seconds
</verification>

<success_criteria>
- BDD test infrastructure is set up using cucumber-rs
- All 9 scenarios from both feature files execute successfully
- Step definitions accurately simulate TapeMeasure component state transitions
- Tests verify core interactions: dragging, momentum, snapping, edge cases, external sync
- Test suite runs via `cargo test --test tape_measure_bdd`
- No compilation errors or warnings
</success_criteria>

<output>
After completion, create `.planning/quick/2-implement-bdd-step-definitions-for-tapem/2-SUMMARY.md`
</output>
