---
phase: quick-10
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/app.rs
  - tests/e2e/tapemeasure.spec.ts
  - tests/e2e/rpe_slider.spec.ts
  - tests/e2e/step_controls.spec.ts
autonomous: true
requirements: []

must_haves:
  truths:
    - "E2E tests wait for WASM hydration before interacting with elements"
    - "Interactive components have data-hydrated attribute after initialization"
    - "Tests pass consistently without timing issues"
  artifacts:
    - path: "src/app.rs"
      provides: "Sets data-hydrated attribute after WASM initialization"
      min_lines: 997
    - path: "tests/e2e/tapemeasure.spec.ts"
      provides: "Waits for data-hydrated before interactions"
      exports: []
    - path: "tests/e2e/rpe_slider.spec.ts"
      provides: "Waits for data-hydrated before interactions"
      exports: []
    - path: "tests/e2e/step_controls.spec.ts"
      provides: "Waits for data-hydrated before interactions"
      exports: []
  key_links:
    - from: "src/app.rs WorkoutInterface component"
      to: "document.body data-hydrated attribute"
      via: "use_effect hook with JS eval"
      pattern: "eval.*setAttribute.*data-hydrated"
    - from: "tests/e2e/*.spec.ts beforeEach"
      to: "document.body[data-hydrated]"
      via: "waitForSelector"
      pattern: "waitForSelector.*data-hydrated"
---

<objective>
Implement hydration-ready pattern to fix E2E test timing issues by adding data-hydrated attribute after WASM initialization and updating all E2E tests to wait for it.

Purpose: Eliminate WASM/Playwright synchronization issues where UI renders but event handlers aren't attached yet, causing 18 E2E test failures.
Output: Reliable E2E tests that wait for complete WASM hydration before interacting with elements.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md

# Current blocker from STATE.md:
# Quick Task 9 PARTIAL: Tests fail due to WASM/Playwright timing issue
# Evidence: Screenshots show UI rendering correctly, issue is purely timing not logic
# Root cause: Playwright can't interact with elements within timeout - needs hydration signal

# Key files involved:
@src/app.rs
@tests/e2e/tapemeasure.spec.ts
@tests/e2e/rpe_slider.spec.ts
@tests/e2e/step_controls.spec.ts
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add data-hydrated attribute to document.body after WASM initialization</name>
  <files>src/app.rs</files>
  <action>
In the `WorkoutInterface` component (starts at line 556), add a `use_effect` hook that runs once on mount to set the `data-hydrated="true"` attribute on `document.body`. This signals that WASM has initialized and event handlers are attached.

Implementation:
1. Add the effect hook after the component's existing logic (around line 559, before the rsx! macro)
2. Use `web_sys::window()` and `document()` to access the DOM
3. Call `set_attribute("data-hydrated", "true")` on the body element
4. Use `use_effect(move || { ... })` with empty dependency array (runs once on mount)
5. Add appropriate error handling with `log::debug!` for success/failure

Why here: WorkoutInterface is the parent component that always renders when the app is Ready (line 455), making it the perfect place to signal that the interactive UI is fully hydrated and ready for interaction.

Pattern to follow:
```rust
use_effect(move || {
    spawn(async move {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    if let Err(e) = body.set_attribute("data-hydrated", "true") {
                        log::error!("Failed to set data-hydrated attribute: {:?}", e);
                    } else {
                        log::debug!("WASM hydration complete - data-hydrated attribute set");
                    }
                }
            }
        }
    });
});
```

Avoid: Don't add to App component (too early - renders during Initializing/SelectingFile states). Don't add to individual components (would set attribute multiple times).
  </action>
  <verify>
    <automated>cargo build --release --target wasm32-unknown-unknown</automated>
  </verify>
  <done>WorkoutInterface component sets data-hydrated="true" on document.body when mounted, visible in browser DevTools after page load</done>
</task>

<task type="auto">
  <name>Task 2: Update E2E test beforeEach hooks to wait for data-hydrated attribute</name>
  <files>tests/e2e/tapemeasure.spec.ts, tests/e2e/rpe_slider.spec.ts, tests/e2e/step_controls.spec.ts</files>
  <action>
Update all three E2E test files to wait for the `data-hydrated` attribute before proceeding with test interactions.

For each file, modify the `beforeEach` hook:
1. After `await page.click('button:has-text("Start Workout")')` (line 27 in tapemeasure.spec.ts)
2. Replace the existing component-specific waitForSelector and 500ms timeout with a single wait for data-hydrated
3. Use `await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 })`
4. This ensures WASM is fully initialized and all event handlers are attached before any test interactions

Pattern for tapemeasure.spec.ts (lines 29-36):
```typescript
// Submit the form (Weighted is already selected by default)
await page.click('button:has-text("Start Workout")');

// Wait for WASM hydration to complete
await page.waitForSelector('body[data-hydrated="true"]', {
  timeout: 10000
});
```

Apply the same pattern to:
- tests/e2e/rpe_slider.spec.ts (lines 29-36)
- tests/e2e/step_controls.spec.ts (lines 29-36)

Remove the component-specific waits and 500ms timeouts - they're no longer needed with the hydration signal.

Why this works: The data-hydrated attribute is set AFTER all Dioxus components have mounted and their event handlers are attached, guaranteeing that Playwright interactions won't race with WASM initialization.
  </action>
  <verify>
    <automated>npm test -- tests/e2e/tapemeasure.spec.ts tests/e2e/rpe_slider.spec.ts tests/e2e/step_controls.spec.ts</automated>
  </verify>
  <done>All 18 E2E tests wait for data-hydrated attribute and pass consistently without timing issues</done>
</task>

</tasks>

<verification>
Overall verification steps:
1. Build succeeds without errors: `cargo build --release --target wasm32-unknown-unknown`
2. All E2E tests pass: `npm test` (18/18 tests passing)
3. No timing-related test failures or timeout errors in test output
4. Browser DevTools shows body[data-hydrated="true"] attribute present after app loads
</verification>

<success_criteria>
1. WorkoutInterface component sets data-hydrated attribute on mount
2. All three E2E test files wait for data-hydrated before interactions
3. E2E tests pass consistently (18/18) without timing issues
4. Test output shows no timeout errors or element accessibility failures
5. Implementation matches the hydration-ready pattern for WASM apps
</success_criteria>

<output>
After completion, create `.planning/quick/10-implement-hydration-ready-pattern-add-da/10-SUMMARY.md`
</output>
