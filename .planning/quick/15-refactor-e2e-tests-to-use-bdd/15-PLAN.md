# Quick Task 15: Refactor E2E tests to use BDD

## Task
1. Install `playwright-bdd` to integrate Cucumber feature files with the Playwright runner.
2. Translate the Playwright spec files (`rpe_slider.spec.ts`, `step_controls.spec.ts`, `tapemeasure.spec.ts`) into Cucumber feature files.
3. Implement step definitions that cover the actions previously done in Playwright.
4. Update `playwright.config.ts` to utilize the BDD test generator.
5. Verify the full test suite passes.
6. Remove old `.spec.ts` E2E test files.