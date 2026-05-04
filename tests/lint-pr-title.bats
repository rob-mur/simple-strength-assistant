#!/usr/bin/env bats
# Tests for scripts/lint-pr-title.sh
# Verifies that the PR-title commitlint check fails on non-conventional titles,
# passes on conventional titles, and skips cleanly when no PR title is in scope.

PROJECT_ROOT="$(cd "$(dirname "$BATS_TEST_FILENAME")/.." && pwd)"
LINT_PR_TITLE="$PROJECT_ROOT/scripts/lint-pr-title.sh"

setup() {
  cd "$PROJECT_ROOT"
}

# ---------------------------------------------------------------------------
# Failing path: non-conventional title
# ---------------------------------------------------------------------------

@test "fails when PR_TITLE is not a valid conventional commit" {
  PR_TITLE="update stuff" run bash "$LINT_PR_TITLE"
  [ "$status" -ne 0 ]
}

@test "failure output identifies the PR title as the offender" {
  PR_TITLE="update stuff" run bash "$LINT_PR_TITLE"
  [ "$status" -ne 0 ]
  [[ "$output" == *"PR title"* ]]
  [[ "$output" == *"update stuff"* ]]
}

# ---------------------------------------------------------------------------
# Passing path: conventional title
# ---------------------------------------------------------------------------

@test "passes when PR_TITLE is a valid conventional commit" {
  PR_TITLE="feat(auth): add session refresh" run bash "$LINT_PR_TITLE"
  [ "$status" -eq 0 ]
  [[ "$output" == *"PR title valid"* ]]
}

@test "passes for each allowed type in .commitlintrc.json" {
  for type in feat fix docs style refactor perf test build ci chore revert; do
    PR_TITLE="$type: example summary" run bash "$LINT_PR_TITLE"
    [ "$status" -eq 0 ] || { echo "Type '$type' rejected unexpectedly: $output"; return 1; }
  done
}

# ---------------------------------------------------------------------------
# Draft/WIP prefix handling
# ---------------------------------------------------------------------------
# Forgejo marks drafts by prefixing the title with "WIP:"; GitLab uses
# "Draft:". Both are removed on undraft, so the underlying title is what
# actually lands on main. Validate against that, not the prefix.

@test "passes when WIP-prefixed title has a valid underlying conventional commit" {
  PR_TITLE="WIP: feat(auth): add session refresh" run bash "$LINT_PR_TITLE"
  [ "$status" -eq 0 ]
  [[ "$output" == *"PR title valid"* ]]
}

@test "passes when Draft-prefixed title has a valid underlying conventional commit" {
  PR_TITLE="Draft: fix: handle empty payload" run bash "$LINT_PR_TITLE"
  [ "$status" -eq 0 ]
  [[ "$output" == *"PR title valid"* ]]
}

@test "fails when WIP-prefixed title has an invalid underlying subject" {
  PR_TITLE="WIP: update stuff" run bash "$LINT_PR_TITLE"
  [ "$status" -ne 0 ]
  [[ "$output" == *"update stuff"* ]] || [[ "$output" == *"WIP: update stuff"* ]]
}

# ---------------------------------------------------------------------------
# Skip path: PR title not in scope
# ---------------------------------------------------------------------------

@test "skips with a visible message when PR_TITLE is unset" {
  unset PR_TITLE
  run bash "$LINT_PR_TITLE"
  [ "$status" -eq 0 ]
  [[ "$output" == *"Skipping PR title"* ]]
  [[ "$output" != *"PR title valid"* ]]
}

@test "skips with a visible message when PR_TITLE is empty" {
  PR_TITLE="" run bash "$LINT_PR_TITLE"
  [ "$status" -eq 0 ]
  [[ "$output" == *"Skipping PR title"* ]]
}

# ---------------------------------------------------------------------------
# Integration: lint.sh invokes the PR-title check
# ---------------------------------------------------------------------------

@test "scripts/lint.sh invokes the PR-title check" {
  grep -q "lint-pr-title.sh" "$PROJECT_ROOT/scripts/lint.sh"
}

# ---------------------------------------------------------------------------
# Workflow wiring: pull_request CI job exposes PR_TITLE to the lint script
# ---------------------------------------------------------------------------

@test "lint job in Forgejo workflow exposes PR_TITLE from pull_request.title" {
  workflow="$PROJECT_ROOT/.forgejo/workflows/ci.yml"
  [ -f "$workflow" ]
  # The lint job (not some other job) must wire PR_TITLE from the pull_request
  # event payload so the script-level check actually receives a title in
  # production. Extract the lint job block and assert PR_TITLE is wired inside
  # it.
  lint_block=$(awk '/^  lint:/{flag=1; next} flag && /^  [a-zA-Z]/{flag=0} flag' "$workflow")
  echo "$lint_block" | grep -q "PR_TITLE:.*github.event.pull_request.title"
}
