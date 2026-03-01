---
phase: quick-9
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - public/db-module.js
  - tests/e2e/tapemeasure.spec.ts
  - tests/e2e/rpe_slider.spec.ts
  - tests/e2e/step_controls.spec.ts
autonomous: true
requirements: []

must_haves:
  truths:
    - "All 18 E2E tests pass when running via ci-test.sh"
    - "Each test gets truly fresh database state (no cross-test contamination)"
    - "Tests can run in parallel without interference"
  artifacts:
    - path: "public/db-module.js"
      provides: "Database cleanup before initialization"
      contains: "db.close()"
    - path: "tests/e2e/*.spec.ts"
      provides: "Test setup with proper timing"
      min_lines: 10
  key_links:
    - from: "tests/e2e/*.spec.ts"
      to: "public/db-module.js"
      via: "initDatabase call in beforeEach"
      pattern: "Create New Database"
---

<objective>
Fix E2E test isolation to achieve 18/18 passing tests by ensuring each test gets a truly fresh database instance.

Purpose: Complete the E2E test infrastructure (started in task 8) by eliminating test contamination from shared database state.

Output: All Playwright E2E tests passing consistently with proper test isolation.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/quick/8-fix-remaining-12-failing-playwright-test/8-SUMMARY.md

## Root Cause Analysis

Task 8 created the storage abstraction (InMemoryStorage) which successfully bypasses OPFS file picker dialogs. However, tests are failing because:

1. **JavaScript database is global:** `public/db-module.js` has `let db = null` at module scope
2. **No cleanup between tests:** When `initDatabase()` is called, it creates a new DB but doesn't close/clear the old one
3. **Shared state across tests:** All tests running in the same page context share the same `db` variable
4. **Result:** Tests find "Bench Press" sessions from previous tests, timeout waiting for fresh UI state

The InMemoryStorage Rust implementation is correct - the issue is on the JavaScript side.

## Current Test Failure Pattern

All 18 tests timeout at the same point:
```
Error: page.fill: Test timeout of 30000ms exceeded.
await page.fill('input[placeholder="Exercise Name"]', 'Test Bench Press');
```

This happens because:
- Tests click "Create New Database"
- BeforeEach tries to create fresh session
- But app loads with existing "Bench Press" session from previous test
- Input field never appears (ActiveSession view is showing instead of SessionPicker)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add database cleanup to initDatabase</name>
  <files>public/db-module.js</files>
  <action>
Modify the `initDatabase` function to close and clear any existing database before creating a new one.

Add cleanup at the start of initDatabase (line 18):
```javascript
export async function initDatabase(fileData) {
    try {
        await ensureSQLLoaded();

        // CRITICAL: Close existing database to ensure test isolation
        // Without this, tests share state and see data from previous tests
        if (db) {
            try {
                db.close();
            } catch (e) {
                console.warn('Failed to close existing database:', e);
            }
            db = null;
        }

        if (fileData && fileData.length > 0) {
            const uint8Array = new Uint8Array(fileData);
            db = new SQL.Database(uint8Array);
        } else {
            db = new SQL.Database();
        }

        return true;
    } catch (error) {
        console.error('Failed to initialize database:', error);
        return false;
    }
}
```

This ensures each call to initDatabase creates a completely fresh database instance with no residual data.
  </action>
  <verify>
    <automated>./scripts/ci-test.sh 2>&1 | grep -E "(passed|failed)" | tail -5</automated>
  </verify>
  <done>initDatabase properly closes old database before creating new one, preventing test contamination</done>
</task>

<task type="auto">
  <name>Task 2: Optimize test setup timing</name>
  <files>tests/e2e/tapemeasure.spec.ts, tests/e2e/rpe_slider.spec.ts, tests/e2e/step_controls.spec.ts</files>
  <action>
Reduce test flakiness by adding explicit waits after database initialization and removing redundant cleanup steps.

For each test file's beforeEach hook:

1. Remove redundant localStorage.clear() after page.reload() (already cleared before goto)
2. Add explicit wait after "Create New Database" click to ensure DB init completes
3. Increase timeout for finishing existing workout (from 2000ms to 3000ms)

Update the beforeEach pattern in all three files:

```javascript
test.beforeEach(async ({ page, context }) => {
  // Force fresh context by clearing storage
  await context.clearCookies();
  await page.goto('/');
  await page.evaluate(() => localStorage.clear());
  await page.waitForLoadState('networkidle');

  // Real user flow: Click "Create New Database" and wait for DB init
  await page.click('text=Create New Database');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(200); // Ensure DB initialization completes

  // If there's already an active session, finish it first
  const finishButton = page.locator('text=Finish Workout Session');
  if (await finishButton.isVisible({ timeout: 3000 }).catch(() => false)) {
    await finishButton.click();
    await page.waitForLoadState('networkidle');
  }

  // Start a workout session
  await page.click('text=Start Session');
  await page.waitForLoadState('networkidle');

  // Fill in exercise name
  await page.fill('input[placeholder="Exercise Name"]', 'Test Bench Press');

  // Select "Weighted" exercise type
  await page.click('text=Weighted');

  // Submit the form
  await page.click('button:has-text("Start Workout")');

  // Wait for ActiveSession to render with component
  await page.waitForSelector('.tape-measure-container, .rpe-slider-container, .step-controls-container', {
    state: 'visible',
    timeout: 10000
  });

  // Allow WASM hydration and event handlers to attach
  await page.waitForTimeout(500);
});
```

Note: The selector patterns differ slightly per file:
- tapemeasure.spec.ts: `.tape-measure-container`
- rpe_slider.spec.ts: `.rpe-slider-container` (or similar)
- step_controls.spec.ts: `.step-controls-container` (or similar)

Check each file for the exact selector used in the existing code.
  </action>
  <verify>
    <automated>./scripts/ci-test.sh 2>&1 | tail -30</automated>
  </verify>
  <done>All 18 E2E tests pass consistently with improved timing and test isolation</done>
</task>

</tasks>

<verification>
Run full CI test suite to verify all tests pass:
```bash
./scripts/ci-test.sh
```

Expected output:
- Cargo tests: 34/34 passing
- BDD tests: 9 scenarios/38 steps passing
- E2E tests: 18/18 passing

If any E2E tests still fail, check:
1. Browser console for database initialization errors
2. Test output for specific timeout/selector issues
3. Whether test-serve process is using test-mode feature
</verification>

<success_criteria>
- [ ] public/db-module.js closes existing database before creating new one
- [ ] All three E2E test files have optimized beforeEach timing
- [ ] ./scripts/ci-test.sh shows 18/18 E2E tests passing
- [ ] No test contamination (each test sees fresh database state)
- [ ] Tests can run in parallel without interference
</success_criteria>

<output>
After completion, create `.planning/quick/9-task-9/9-SUMMARY.md` documenting:
- The root cause (global JS db variable without cleanup)
- The fix (db.close() before new Database())
- Final test results (18/18 passing)
- Any remaining quirks or timing considerations
</output>
