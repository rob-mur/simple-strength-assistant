---
phase: quick-6
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - devenv.nix
autonomous: true
requirements: []

must_haves:
  truths:
    - "Playwright can find Chromium browser in devenv shell"
    - "ci-test.sh runs successfully with Playwright tests"
    - "No browser download required when running npx playwright install"
  artifacts:
    - path: "devenv.nix"
      provides: "Chromium browser package"
      contains: "chromium"
  key_links:
    - from: "devenv.nix"
      to: "Playwright tests"
      via: "PLAYWRIGHT_BROWSERS_PATH environment variable"
      pattern: "PLAYWRIGHT_BROWSERS_PATH|chromium"
---

<objective>
Add Chromium to devenv.nix to provide Playwright with browser dependencies, eliminating the need for manual browser installation and fixing CI test execution.

Purpose: Playwright tests currently fail or require manual browser installation because the Chromium browser and its system dependencies are not available in the devenv shell. NixOS requires explicit package declaration.

Output: Updated devenv.nix with Chromium configured and environment variables set for Playwright to use the Nix-provided browser.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@devenv.nix
@playwright.config.ts
@scripts/ci-test.sh

## Background

From STATE.md:
- Quick Task 3 implemented Playwright E2E tests (18 tests across 3 components)
- Note states: "E2E tests production-ready but can't run in NixOS (environmental limitation)"
- Quick Task 4 updated ci-test.sh to use devenv processes

The issue: Playwright requires Chromium browser binaries and system dependencies. In NixOS/devenv, these must be explicitly declared as packages. The standard `npx playwright install` approach doesn't work reliably in Nix environments.

## Current devenv.nix structure

```nix
packages = with pkgs; [
  git
  gh
  dioxus-cli
  wasm-bindgen-cli
  binaryen
  devcontainer
  claude-code
  gemini-cli-bin
];
```

No Chromium or browser-related packages currently included.

## Playwright configuration

Tests use two projects:
- `chromium` (Desktop Chrome)
- `Mobile Safari` (iPhone 13 simulation - uses Chromium engine)

Base URL: http://localhost:8080 (served via devenv processes)
</context>

<tasks>

<task type="auto">
  <name>Add Chromium to devenv.nix packages and set Playwright environment</name>
  <files>devenv.nix</files>
  <action>
Add Chromium to the packages list in devenv.nix:

1. Add `chromium` to the packages array (after `gemini-cli-bin`)

2. Add environment variables section after the `languages.javascript` block to point Playwright at the Nix-provided Chromium:

```nix
env = {
  PLAYWRIGHT_BROWSERS_PATH = "${pkgs.chromium}";
  PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
};
```

This tells Playwright to:
- Use the Chromium binary from Nix packages (not download its own)
- Skip the normal browser download process

The Nix package includes all system dependencies Chromium needs (libraries, fonts, etc.), which is why this approach is more reliable than `playwright install` in NixOS.

Note: Do NOT use `playwright-driver` package - it's outdated and incompatible with @playwright/test 1.48.0. The chromium package provides the browser binary that Playwright expects.
  </action>
  <verify>
```bash
# Verify chromium is in packages
grep -q "chromium" devenv.nix && echo "PASS: chromium in devenv.nix" || echo "FAIL"

# Verify environment variables are set
grep -q "PLAYWRIGHT_BROWSERS_PATH" devenv.nix && echo "PASS: env vars configured" || echo "FAIL"

# After reloading devenv shell, verify Playwright can see browser
devenv shell -- npx playwright install --dry-run 2>&1 | grep -q "chromium.*already installed" || echo "Browser accessible"
```
  </verify>
  <done>
- devenv.nix includes chromium in packages list
- Environment variables PLAYWRIGHT_BROWSERS_PATH and PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD are set
- File changes committed
  </done>
</task>

<task type="auto">
  <name>Test Playwright with devenv-provided Chromium</name>
  <files></files>
  <action>
Reload the devenv environment and run the CI test script to verify Playwright tests execute successfully with the Nix-provided Chromium:

1. Exit and re-enter devenv shell to pick up new packages and environment variables
2. Run `devenv ci-test` or `./scripts/ci-test.sh`
3. Verify all Playwright tests execute (should see chromium project tests running)
4. Check test output confirms browser launched successfully

Expected output:
- No "Executable doesn't exist" errors
- Chromium tests run (18 tests across TapeMeasure, RPESlider, StepControls)
- Mobile Safari tests run (uses Chromium engine for iPhone 13 simulation)
- Tests either pass or fail on assertions (browser availability not the blocker)

If tests fail on assertions (not browser availability), that's acceptable - the goal is browser accessibility, not test correctness.
  </action>
  <verify>
```bash
# Run the full CI test suite
./scripts/ci-test.sh
```

Success criteria:
- Script completes without browser-not-found errors
- Playwright test runner launches
- Browser instances start (visible in output logs)
  </verify>
  <done>
- CI test script runs to completion
- Playwright successfully launches Chromium browser
- No browser installation or dependency errors
- Test execution demonstrates browser availability
  </done>
</task>

</tasks>

<verification>
After plan completion:

1. **devenv.nix updated:** chromium package added, environment variables configured
2. **Playwright integration:** Tests run without browser installation errors
3. **CI pipeline:** scripts/ci-test.sh completes successfully with E2E tests

Manual verification:
```bash
# In devenv shell
echo $PLAYWRIGHT_BROWSERS_PATH  # Should show chromium path
npx playwright test --project=chromium  # Should run without browser errors
```
</verification>

<success_criteria>
- devenv.nix includes chromium package
- Environment variables PLAYWRIGHT_BROWSERS_PATH and PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD are set
- ci-test.sh runs Playwright tests without browser availability errors
- E2E tests can execute in NixOS environment (resolving "can't run in NixOS" limitation from STATE.md)
</success_criteria>

<output>
After completion, create `.planning/quick/6-add-chromium-to-devenv-to-fix-playwright/6-SUMMARY.md`
</output>
