# Quick Task 14: Fix lints and investigate CI run timeout/exit code issue

## Task
1. Investigate the latest CI failure to see if it was due to another Playwright timeout.
2. Fix the timeout by increasing the overall Playwright timeout limit when running in the CI environment.
3. Investigate the lint failure. The `ci-test.sh` script is running `npx commitlint` for the PR's commits, and some commits from earlier in Phase 6 still have uppercase subjects.
4. Rewrite those commit messages via an interactive rebase.
5. Ensure `npx commitlint` passes for all commits on the branch against `origin/main`.