---
phase: quick-5
plan: 5
type: execute
wave: 1
depends_on: []
files_modified:
  - src/components/tape_measure.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "All commitlint validations pass"
    - "Clippy passes with no warnings"
    - "CI test script executes successfully"
  artifacts:
    - path: "src/components/tape_measure.rs"
      provides: "Fixed clippy warning"
      min_lines: 200
  key_links:
    - from: "scripts/lint.sh"
      to: "git commit history"
      via: "commitlint validation"
      pattern: "npx commitlint"
---

<objective>
Fix linting and CI test issues to allow clean git commits and CI runs.

Purpose: Resolve commitlint violations in recent commit messages and fix clippy warning in TapeMeasure component.
Output: Clean lint and CI test passes, enabling git hooks to succeed.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

## Current Issues Identified

### 1. Commitlint Violations
Two commits violate conventional commit rules:
- `5338b09` - header too long (120 chars, max 100)
- `c81dfbf` - subject in sentence-case (must be lowercase)
- `3fe53cf` - body line too long (>100 chars)

### 2. Clippy Warning
In `src/components/tape_measure.rs:227`:
```
error: using `clone` on type `Signal<bool>` which implements the `Copy` trait
let mut click_allowed_clone = click_allowed.clone();
                               ^^^^^^^^^^^^^^^^^^^^^
```

Fix: Replace `.clone()` with direct copy since `Signal<bool>` implements `Copy`.

## CI Steps (from devenv.nix)
The git-hooks run in order:
1. `format` - cargo fmt (passing)
2. `lint` - ./scripts/lint.sh (failing on commitlint + clippy)
3. `ci-test` - ./scripts/ci-test.sh
4. `build` - dx bundle
</context>

<tasks>

<task type="auto">
  <name>Task 1: Fix clippy warning in TapeMeasure</name>
  <files>src/components/tape_measure.rs</files>
  <action>
Fix the clippy::clone_on_copy warning at line 227. Replace `click_allowed.clone()` with just `click_allowed` since `Signal<bool>` implements the Copy trait.

Location: Line 227 in the pointer event handler.
Change from: `let mut click_allowed_clone = click_allowed.clone();`
Change to: `let mut click_allowed_clone = click_allowed;`
  </action>
  <verify>
    <automated>cargo clippy -- -D warnings</automated>
  </verify>
  <done>Clippy passes with zero warnings</done>
</task>

<task type="auto">
  <name>Task 2: Reword problematic commit messages</name>
  <files></files>
  <action>
Use interactive rebase to fix three commits with commitlint violations:

1. Reword `5338b09` (HEAD~4):
   From: "docs(quick-4): please add playwright tests to ci-test script. for any necessary background services use devenv processes"
   To: "docs(quick-4): add playwright tests to ci-test script with devenv processes"

2. Reword `c81dfbf` (HEAD~7):
   From: "docs(quick-3): Address PR review comments and implement Playwright E2E tests"
   To: "docs(quick-3): address PR review comments and implement Playwright E2E tests"

3. Edit `3fe53cf` (HEAD~10) body to wrap long lines at 100 chars:
   Keep subject: "feat(quick-3): implement Playwright E2E tests for tactile components"
   Rewrap body lines to stay within 100 character limit.

Execute: `git rebase -i HEAD~11`
Mark the three commits for 'reword' or 'edit', fix messages, continue rebase.
  </action>
  <verify>
    <automated>npx commitlint --from main --to HEAD --verbose</automated>
  </verify>
  <done>All commits from main to HEAD pass commitlint with zero errors</done>
</task>

<task type="auto">
  <name>Task 3: Verify full CI pipeline</name>
  <files></files>
  <action>
Run the complete CI pipeline to ensure all checks pass:

1. Format check: `cargo fmt -- --check`
2. Lint check: `./scripts/lint.sh`
3. CI tests: `./scripts/ci-test.sh` (cargo test + playwright via devenv processes)
4. Build: `dx bundle --web --release --debug-symbols=false`

All must pass with zero errors. The git hooks will now succeed on commit.
  </action>
  <verify>
    <automated>./scripts/lint.sh && ./scripts/ci-test.sh</automated>
  </verify>
  <done>Full lint and CI test pipeline passes successfully</done>
</task>

</tasks>

<verification>
1. Clippy produces zero warnings
2. All commits pass commitlint validation
3. Full CI pipeline (format, lint, ci-test, build) succeeds
4. Git pre-commit hooks can run without failures
</verification>

<success_criteria>
- Clippy warning eliminated from tape_measure.rs
- All commit messages conform to conventional commit format
- scripts/lint.sh exits with code 0
- scripts/ci-test.sh completes successfully
- Git hooks ready for clean commits
</success_criteria>

<output>
After completion, create `.planning/quick/5-please-fix-the-lints-and-ci-tests-see-th/5-SUMMARY.md`
</output>
