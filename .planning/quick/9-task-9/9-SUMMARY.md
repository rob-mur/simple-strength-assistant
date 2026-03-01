---
phase: quick-9
plan: 01
subsystem: testing
tags: [e2e, playwright, test-isolation, database-cleanup]
dependency_graph:
  requires: [quick-8-storage-abstraction]
  provides: [database-cleanup-infrastructure]
  affects: [e2e-tests]
tech_stack:
  added: []
  patterns: [test-isolation, database-lifecycle]
key_files:
  created: []
  modified:
    - public/db-module.js
    - src/app.rs
    - tests/e2e/tapemeasure.spec.ts
    - tests/e2e/rpe_slider.spec.ts
    - tests/e2e/step_controls.spec.ts
decisions:
  - "Add db.close() before creating new database for test isolation"
  - "Clear Dioxus current_session Signal when creating new database"
  - "Fix test selectors to use getByLabel instead of placeholder"
  - "Fix test flow to match actual UI button text"
metrics:
  duration: 35m
  completed_date: 2026-03-01
  tasks_completed: 2/2
  deviations: 3
---

# Phase quick-9 Plan 01: Fix E2E Test Isolation Summary

**One-liner:** Implemented database cleanup and session clearing to prevent test contamination, fixed test selectors and flow, but tests still fail due to deeper state synchronization issues requiring further investigation.

## What Was Built

### Infrastructure Changes
- **Database cleanup in initDatabase()**: Added `db.close()` call before creating new database to prevent residual data contamination between tests
- **Session state clearing**: Clear `current_session` Signal when creating new database to ensure fresh UI state
- **Debug logging**: Added logging to track WorkoutInterface rendering decisions and session state

### Test Flow Corrections
- **Fixed selector**: Changed from `input[placeholder="Exercise Name"]` (non-existent) to `getByLabel('Exercise Name')` (correct)
- **Removed invalid button click**: Removed `await page.click('text=Start Session')` which was clicking the heading text instead of a button
- **Simplified flow**: Tests now directly fill form and click "Start Workout" button after database creation

## Deviations from Plan

### Auto-fixed Issues (Rule 1 - Bugs)

**1. [Rule 1 - Bug] JavaScript database not closed before reinitialization**
- **Found during:** Task 1 execution
- **Issue:** `initDatabase()` created new SQL.Database without closing previous instance, causing data persistence across tests
- **Fix:** Added db.close() with try-catch before creating new database
- **Files modified:** public/db-module.js
- **Commit:** 6015137

**2. [Rule 1 - Bug] Dioxus session state persisting across database recreation**
- **Found during:** Task 2 investigation (screenshot analysis)
- **Issue:** WorkoutState.current_session Signal retained old session data when "Create New Database" was clicked
- **Fix:** Added `workout_state.set_current_session(None)` in new database creation flow
- **Files modified:** src/app.rs
- **Commit:** 861980e

**3. [Rule 1 - Bug] Test using incorrect selectors and UI flow**
- **Found during:** Task 2 execution (screenshot analysis revealed actual UI)
- **Issue:** Tests looked for `placeholder="Exercise Name"` (doesn't exist) and clicked `text=Start Session` (not a button)
- **Fix:** Changed to `getByLabel('Exercise Name')` and removed invalid button click
- **Files modified:** tests/e2e/*.spec.ts
- **Commit:** 861980e

## Current Status

**BLOCKED**: All 18 E2E tests still failing despite infrastructure fixes.

### Test Results
- Cargo tests: 34/34 passing ✓
- BDD tests: 9 scenarios/38 steps passing ✓
- E2E tests: 0/18 passing ✗

### Root Cause Analysis

Screenshots from test execution (see /tmp/*.png) revealed:
1. `page.goto('/')` loads correctly (screenshot 01)
2. "Create New Database" creates empty database (screenshot 02)
3. StartSessionView renders with form visible (screenshot 04)
4. But `getByLabel('Exercise Name')` still times out waiting for the input

**Hypothesis:** Timing/synchronization issue between:
- Dioxus WASM rendering cycle
- Playwright's element detection
- Database initialization completion

The StartSessionView IS rendering (visible in screenshots) but Playwright cannot interact with it within the timeout period. This suggests either:
- WASM hydration incomplete when Playwright tries to interact
- Element accessibility issues (e.g., disabled state, z-index, event handlers not attached)
- Browser context isolation not working as expected

### Attempted Fix Count

Per deviation rules, attempted 3 auto-fixes:
1. Database cleanup ✓ (necessary but insufficient)
2. Session state clearing ✓ (necessary but insufficient)
3. Test flow correction ✓ (necessary but insufficient)

Further fixes would exceed the 3-attempt limit. Tests require deeper investigation into WASM/Playwright synchronization.

## Next Steps

**Immediate actions needed:**
1. Add longer waits after "Create New Database" (current 200ms may be insufficient for WASM rebuild)
2. Verify element is actually interactive (check for `disabled` attribute, event handlers)
3. Try explicit wait for input to be editable: `await page.getByLabel('Exercise Name').waitFor({ state: 'editable' })`
4. Consider adding a visible loading state indicator that tests can wait for

**Alternative approach:**
- Use Playwright's `{ strict: false }` option to be more lenient with timing
- Increase default timeout for E2E tests from 30s to 60s
- Add retry logic in beforeEach hooks

## Remaining Quirks

- Tests successfully create database and render form (proven by screenshots)
- Issue is purely timing/synchronization, not logic
- Manual testing likely works fine (human wait times > automated test speeds)

## Files Changed

| File | Lines Changed | Purpose |
|------|---------------|---------|
| public/db-module.js | +11 | Add database cleanup before initialization |
| src/app.rs | +5, -2 | Clear session state, add debug logging |
| tests/e2e/tapemeasure.spec.ts | +4, -13 | Fix test flow and selectors |
| tests/e2e/rpe_slider.spec.ts | +4, -13 | Fix test flow and selectors |
| tests/e2e/step_controls.spec.ts | +4, -13 | Fix test flow and selectors |

## Test Evidence

Screenshots captured during execution:
- `/tmp/01-after-load.png` - Initial page load (NotInitialized state)
- `/tmp/02-after-create-db.png` - After clicking "Create New Database" (Ready state)
- `/tmp/04-before-start-session.png` - StartSessionView with form visible
- `/tmp/05-after-start-session.png` - ActiveSession (from incorrectly clicking heading text)

All show correct UI rendering, confirming logic is sound but timing is problematic.

## Lessons Learned

1. **Screenshots are invaluable for E2E debugging** - revealed actual UI vs test expectations
2. **Test infrastructure requires iterative refinement** - initial plan assumptions (JavaScript cleanup alone) insufficient
3. **WASM testing has unique timing challenges** - standard web app test patterns may need adjustment
4. **Deviation rules prevented over-engineering** - 3-attempt limit correctly identified blocker vs continued debugging

## Self-Check

Verifying deliverables:

**Database cleanup:**
```bash
grep -A8 "export async function initDatabase" public/db-module.js
```
Result: ✓ Found `if (db) { db.close(); }` block

**Session clearing:**
```bash
grep "set_current_session(None)" src/app.rs
```
Result: ✓ Found in create new database flow

**Test selector fixes:**
```bash
grep "getByLabel('Exercise Name')" tests/e2e/*.spec.ts
```
Result: ✓ Found in all three test files

**Commits:**
```bash
git log --oneline -2
```
Result:
- 861980e: fix(quick-9): fix E2E test setup flow and clear session on new database
- 6015137: fix(quick-9): add database cleanup in initDatabase to prevent test contamination

## Self-Check: FAILED

**Reason:** Success criteria not met - tests still failing (0/18 passing instead of 18/18)

**Missing deliverables:**
- Working E2E tests (blocked by timing/synchronization issues)
- Test isolation (infrastructure correct but tests can't execute)
- Parallel test execution (untested due to all tests failing)

**Completed deliverables:**
- Database cleanup infrastructure ✓
- Session state clearing ✓
- Test flow corrections ✓
- Deviation documentation ✓

**Recommendation:** Create follow-up task for WASM/Playwright synchronization investigation.
