---
phase: quick-3
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/components/tape_measure.rs
  - src/components/step_controls.rs
  - tests/e2e/tapemeasure.spec.ts
  - tests/e2e/rpe_slider.spec.ts
  - tests/e2e/step_controls.spec.ts
  - playwright.config.ts
  - package.json
  - .planning/phases/06-jump-controls/06-UAT.md.bak
  - .claude/commands/gsd/new-project.md.bak
  - .gemini/commands/gsd/new-project.md.bak
autonomous: true
requirements: []

must_haves:
  truths:
    - "Critical bugs fixed: onmounted downcast, ghost clicks, NaN panic"
    - "Code quality improved: float formatting, consistent comparisons, redundancy removed"
    - "Playwright E2E tests verify real DOM interactions for all tactile components"
  artifacts:
    - path: "src/components/tape_measure.rs"
      provides: "Ghost click prevention on drag release"
      contains: "click_allowed"
    - path: "src/components/step_controls.rs"
      provides: "NaN-safe sorting with total_cmp"
      contains: "total_cmp"
    - path: "playwright.config.ts"
      provides: "Playwright configuration for E2E tests"
      exports: ["defineConfig"]
    - path: "tests/e2e/tapemeasure.spec.ts"
      provides: "E2E tests for TapeMeasure component"
      contains: "test('swipe drag"
  key_links:
    - from: "src/components/tape_measure.rs"
      to: "container_element signal"
      via: "onmounted Element type check"
      pattern: "if let Some\\(raw\\) = el\\.data\\.downcast::<web_sys::Element>"
    - from: "tests/e2e/*.spec.ts"
      to: "http://localhost:8080"
      via: "Playwright browser automation"
      pattern: "page.goto"
---

<objective>
Address all PR review comments: fix critical bugs (onmounted downcast, ghost clicks, NaN panic), improve code quality (float formatting, consistent comparisons, remove redundancy), remove .bak files, and implement Playwright E2E tests to verify real DOM interactions for TapeMeasure, RPESlider, and StepControls components.

Purpose: Ensure production-ready code quality and comprehensive E2E test coverage beyond BDD physics simulations.

Output:
- Bug-free tactile components with proper DOM handling
- Playwright test suite with E2E coverage
- Clean repository (no .bak files)
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md

## Key Interfaces

From src/components/tape_measure.rs:
```rust
#[derive(Props, PartialEq, Clone)]
pub struct TapeMeasureProps {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub on_change: EventHandler<f64>,
}
```

From src/components/step_controls.rs:
```rust
#[derive(Props, PartialEq, Clone)]
pub struct StepControlsProps {
    pub value: f64,
    pub steps: Vec<f64>,
    pub min: f64,
    pub max: f64,
    pub on_change: EventHandler<f64>,
}
```

From src/components/rpe_slider.rs:
```rust
#[derive(Props, PartialEq, Clone)]
pub struct RPESliderProps {
    pub value: f64,
    pub on_change: EventHandler<f64>,
}
```
</context>

<tasks>

<task type="auto">
  <name>Task 1: Fix critical bugs and code quality issues</name>
  <files>
    src/components/tape_measure.rs
    src/components/step_controls.rs
    .planning/phases/06-jump-controls/06-UAT.md.bak
    .claude/commands/gsd/new-project.md.bak
    .gemini/commands/gsd/new-project.md.bak
  </files>
  <action>
**Critical Bugs:**

1. **Fix onmounted downcast (tape_measure.rs line 132-136):**
   - Current code: `if let Some(raw) = el.data.downcast::<web_sys::Element>()`
   - Issue: Dioxus MountedData may provide HtmlElement, not Element
   - Fix: Try both types: `downcast::<web_sys::HtmlElement>()` first, then fallback to `Element`
   - Add debug logging if both fail to diagnose in production

2. **Fix ghost clicks on drag release (tape_measure.rs line 257-262):**
   - Issue: Swipe gestures trigger the onclick handler in tick hitboxes
   - Solution: Add click suppression using a signal `click_allowed` (Signal<bool>)
   - Set `click_allowed.set(false)` in onpointerdown
   - Set `click_allowed.set(true)` after 200ms delay in onpointerup (only if drag distance was < 5px)
   - Wrap onclick handler: `if !click_allowed() { return; }`
   - Use gloo_timers::future::TimeoutFuture for the delay

3. **Fix NaN panic in StepControls (step_controls.rs line 25-26):**
   - Current: `a.partial_cmp(b).unwrap()`
   - Issue: Panics if Vec contains NaN
   - Fix: Replace with `a.total_cmp(b)` (Rust 1.62+, handles NaN deterministically)

**Code Quality:**

4. **Float display precision in TapeMeasure (tape_measure.rs line 282):**
   - Current: `"{val}"`
   - Issue: May show excessive decimals (e.g., "2.5000000000001")
   - Fix: Use conditional formatting based on step size:
     - If `props.step >= 1.0`: `"{val:.0}"` (no decimals)
     - If `props.step >= 0.1`: `"{val:.1}"` (1 decimal)
     - Else: `"{val:.2}"` (2 decimals)

5. **Consistent float comparison (tape_measure.rs lines 251, 59, 81):**
   - Current: Mix of exact `!=` (line 251) and epsilon `>= VELOCITY_THRESHOLD + f64::EPSILON` (line 59)
   - Fix: Use epsilon tolerance consistently:
     - Line 251: `(val % (props.step * 2.0)).abs() < f64::EPSILON || props.step >= 1.0`
     - Line 59: Already correct (keep as-is)
     - Line 81: Already correct (keep as-is)
   - Add const `EPSILON_TOLERANCE: f64 = 1e-9;` at top of file, use instead of `f64::EPSILON` where appropriate

6. **Remove redundant style (step_controls.rs line 31):**
   - Current: `style: "width: 100%;"`
   - Issue: Already has `class: "... w-full ..."`
   - Fix: Delete the style attribute entirely

**Cleanup:**

7. **Delete .bak files:**
   - Remove all three .bak files identified by glob:
     - `.planning/phases/06-jump-controls/06-UAT.md.bak`
     - `.claude/commands/gsd/new-project.md.bak`
     - `.gemini/commands/gsd/new-project.md.bak`

**Performance (Deferred):**
- Issue #8 (use_future RAF-based loop) is a valid optimization but requires architectural changes
- Document in code comment: "TODO: Consider RAF-based animation loop instead of 16ms timer for battery efficiency"
- Address in future refactor (not blocking for this PR)
  </action>
  <verify>
```bash
# Compile check
cargo check

# Run existing BDD tests
cargo test --test tape_measure_bdd

# Verify .bak files removed
! ls .planning/phases/06-jump-controls/06-UAT.md.bak 2>/dev/null && \
! ls .claude/commands/gsd/new-project.md.bak 2>/dev/null && \
! ls .gemini/commands/gsd/new-project.md.bak 2>/dev/null && \
echo "All .bak files removed successfully" || echo "ERROR: .bak files still exist"
```
  </verify>
  <done>
- All critical bugs fixed (onmounted, ghost clicks, NaN panic)
- Code quality improved (float formatting, consistent comparisons, no redundancy)
- All .bak files removed
- Cargo check passes
- Existing BDD tests pass
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement Playwright E2E tests</name>
  <files>
    package.json
    playwright.config.ts
    tests/e2e/tapemeasure.spec.ts
    tests/e2e/rpe_slider.spec.ts
    tests/e2e/step_controls.spec.ts
    .gitignore
  </files>
  <action>
**Setup Playwright:**

1. **Add Playwright to package.json:**
   - Add to devDependencies: `"@playwright/test": "^1.48.0"`
   - Add to devDependencies: `"@types/node": "^22.10.2"` (for TypeScript support)
   - Add test script: `"test:e2e": "playwright test"`
   - Add test:ui script: `"test:e2e:ui": "playwright test --ui"`
   - Add test:headed script: `"test:e2e:headed": "playwright test --headed"`

2. **Create playwright.config.ts:**
   ```typescript
   import { defineConfig, devices } from '@playwright/test';

   export default defineConfig({
     testDir: './tests/e2e',
     fullyParallel: true,
     forbidOnly: !!process.env.CI,
     retries: process.env.CI ? 2 : 0,
     workers: process.env.CI ? 1 : undefined,
     reporter: 'html',
     use: {
       baseURL: 'http://localhost:8080',
       trace: 'on-first-retry',
     },
     projects: [
       {
         name: 'chromium',
         use: { ...devices['Desktop Chrome'] },
       },
       {
         name: 'Mobile Safari',
         use: { ...devices['iPhone 13'] },
       },
     ],
     webServer: {
       command: 'dx serve --port 8080',
       url: 'http://localhost:8080',
       reuseExistingServer: !process.env.CI,
     },
   });
   ```

3. **Create tests/e2e/tapemeasure.spec.ts:**
   Test real DOM interactions that BDD tests cannot verify:
   - Swipe drag gesture (pointer events, not simulated physics)
   - Click-to-jump on tick marks (actual DOM clicks)
   - Pointer capture behavior (browser-native, not simulatable)
   - SVG rendering and transform updates
   - Visual snapping animation (requestAnimationFrame timing)

   Example test structure:
   ```typescript
   test('swipe drag updates value', async ({ page }) => {
     await page.goto('/active-session'); // Adjust to actual route
     const tape = page.locator('[class*="tape-measure-container"]').first();

     // Get initial value from SVG text
     const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

     // Perform swipe gesture
     await tape.dispatchEvent('pointerdown', { clientX: 150, clientY: 40 });
     await tape.dispatchEvent('pointermove', { clientX: 100, clientY: 40 });
     await tape.dispatchEvent('pointerup', { clientX: 100, clientY: 40 });

     // Wait for snap animation
     await page.waitForTimeout(500);

     // Verify value changed
     const finalValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
     expect(finalValue).not.toBe(initialValue);
   });
   ```

4. **Create tests/e2e/rpe_slider.spec.ts:**
   - Test HTML range input interaction (not covered by BDD)
   - Verify color class changes on value update
   - Test oninput handler triggers correctly
   - Verify legend text updates

5. **Create tests/e2e/step_controls.spec.ts:**
   - Test button click handlers
   - Verify glass effect rendering
   - Test value clamping at boundaries
   - Verify SVG icon rendering

6. **Update .gitignore:**
   Add Playwright artifacts:
   ```
   /test-results/
   /playwright-report/
   /playwright/.cache/
   ```

**Coverage Requirements:**
- Each component gets its own spec file
- Each spec tests at least 3 scenarios (happy path, edge case, visual verification)
- Tests run against real Dioxus dev server (dx serve)
- Both desktop and mobile viewport configs
  </action>
  <verify>
```bash
# Install dependencies
npm install

# Install Playwright browsers
npx playwright install chromium

# Run E2E tests (requires dev server)
# Start dev server in background if not running
if ! curl -s http://localhost:8080 > /dev/null; then
  dx serve --port 8080 &
  SERVER_PID=$!
  sleep 5
fi

# Run tests
npm run test:e2e

# Cleanup
if [ ! -z "$SERVER_PID" ]; then
  kill $SERVER_PID
fi
```
  </verify>
  <done>
- Playwright installed and configured
- Three E2E test files created (tapemeasure, rpe_slider, step_controls)
- Tests verify real DOM interactions (pointer events, clicks, rendering)
- Tests pass against running dev server
- .gitignore updated with Playwright artifacts
  </done>
</task>

</tasks>

<verification>
**Overall Phase Verification:**

1. **Code Quality:**
   ```bash
   cargo clippy -- -D warnings
   cargo fmt -- --check
   ```

2. **All Tests Pass:**
   ```bash
   # BDD tests
   cargo test

   # E2E tests
   npm run test:e2e
   ```

3. **Build Success:**
   ```bash
   dx build --release
   ```

4. **Manual Verification:**
   - Deploy to Vercel preview
   - Test TapeMeasure swipe on mobile device
   - Verify no ghost clicks when releasing drag
   - Verify StepControls buttons work with extreme values (NaN boundary)
</verification>

<success_criteria>
1. All 7 PR review issues resolved (bugs fixed, code quality improved, .bak files removed)
2. Playwright E2E test suite implemented with ≥9 test scenarios across 3 components
3. All tests (BDD + E2E) pass
4. Code compiles with no warnings
5. Visual verification confirms ghost click fix on real device
</success_criteria>

<output>
After completion, create `.planning/quick/3-address-pr-review-comments-and-implement/3-SUMMARY.md`
</output>
