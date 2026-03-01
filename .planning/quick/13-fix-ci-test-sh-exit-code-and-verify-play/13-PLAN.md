# Quick Task 13: Fix ci-test.sh exit code and verify Playwright test failures are captured in CI

## Task
1. Fix `scripts/ci-test.sh` to correctly propagate the exit code from `npm run test:e2e`. Currently, `trap "devenv processes down" EXIT` might be causing the script to exit with 0 even if Playwright fails.
2. Verify the fix by temporarily forcing a Playwright test failure and ensuring `ci-test.sh` returns a non-zero exit code.
3. Ensure that the original timeout issues (if any) are also resolved/stable.

## Proposed Changes
- Capture the exit code of `npm run test:e2e` in `scripts/ci-test.sh` and explicitly exit with it.
- Alternatively, adjust the `trap` to ensure it doesn't clobber the exit status.