#!/usr/bin/env bats
# Tests for agent-facing minimal-output wrapper scripts
# These scripts suppress verbose output to reduce LLM context consumption.

SCRIPTS_DIR="$(cd "$(dirname "$BATS_TEST_FILENAME")/../scripts" && pwd)"

# ---------------------------------------------------------------------------
# agent-test.sh
# ---------------------------------------------------------------------------

@test "agent-test.sh exists and is executable" {
  [ -x "$SCRIPTS_DIR/agent-test.sh" ]
}

@test "agent-test.sh outputs only 'Passed' on a green test run" {
  _BATS_CARGO_TEST_STUB=pass _BATS_NPM_E2E_STUB=pass run bash "$SCRIPTS_DIR/agent-test.sh"
  [ "$status" -eq 0 ]
  [ "$output" = "Passed" ]
}

@test "agent-test.sh exits non-zero and outputs failing info when cargo test fails" {
  _BATS_CARGO_TEST_STUB=fail _BATS_NPM_E2E_STUB=skip run bash "$SCRIPTS_DIR/agent-test.sh"
  [ "$status" -ne 0 ]
  [ -n "$output" ]
  [[ "$output" != *"Running"* ]]
  [[ "$output" != *"Waiting"* ]]
}

@test "agent-test.sh exits non-zero and outputs failing info when e2e tests fail" {
  _BATS_CARGO_TEST_STUB=pass _BATS_NPM_E2E_STUB=fail run bash "$SCRIPTS_DIR/agent-test.sh"
  [ "$status" -ne 0 ]
  [ -n "$output" ]
  [[ "$output" != *"Waiting for"* ]]
}

@test "agent-test.sh skips e2e tests when cargo tests fail" {
  _BATS_CARGO_TEST_STUB=fail _BATS_NPM_E2E_STUB=pass run bash "$SCRIPTS_DIR/agent-test.sh"
  [ "$status" -ne 0 ]
  [[ "$output" != *"Scenario"* ]]
}

# ---------------------------------------------------------------------------
# agent-lint.sh
# ---------------------------------------------------------------------------

@test "agent-lint.sh exists and is executable" {
  [ -x "$SCRIPTS_DIR/agent-lint.sh" ]
}

@test "agent-lint.sh outputs only 'Passed' when all lint checks pass" {
  _BATS_LINT_STUB=pass run bash "$SCRIPTS_DIR/agent-lint.sh"
  [ "$status" -eq 0 ]
  [ "$output" = "Passed" ]
}

@test "agent-lint.sh exits non-zero and outputs only error lines when lint fails" {
  _BATS_LINT_STUB=fail run bash "$SCRIPTS_DIR/agent-lint.sh"
  [ "$status" -ne 0 ]
  [ -n "$output" ]
  [[ "$output" != *"Running linting"* ]]
  [[ "$output" != *"→ Checking"* ]]
  [[ "$output" != *"→ Running"* ]]
  [[ "$output" != *"✓"* ]]
}

# ---------------------------------------------------------------------------
# agent-fmt.sh
# ---------------------------------------------------------------------------

@test "agent-fmt.sh exists and is executable" {
  [ -x "$SCRIPTS_DIR/agent-fmt.sh" ]
}

@test "agent-fmt.sh outputs 'Passed' and exits 0 when formatting is clean" {
  _BATS_FMT_STUB=pass run bash "$SCRIPTS_DIR/agent-fmt.sh"
  [ "$status" -eq 0 ]
  [ "$output" = "Passed" ]
}

@test "agent-fmt.sh exits non-zero and lists unformatted files when formatting is needed" {
  _BATS_FMT_STUB=fail run bash "$SCRIPTS_DIR/agent-fmt.sh"
  [ "$status" -ne 0 ]
  [[ "$output" == *"src/main.rs"* ]]
  [[ "$output" != *"Checking"* ]]
  [[ "$output" != *"✓"* ]]
}

# ---------------------------------------------------------------------------
# Hook router script
# ---------------------------------------------------------------------------

HOOK="$(dirname "$SCRIPTS_DIR")/.claude/hooks/agent-command-router.sh"

@test "agent-command-router.sh exists and is executable" {
  [ -x "$HOOK" ]
}

@test "agent-command-router.sh rewrites cargo test to agent-test.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"cargo test"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-test.sh"* ]]
}

@test "agent-command-router.sh rewrites npm run test:e2e to agent-test.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"npm run test:e2e"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-test.sh"* ]]
}

@test "agent-command-router.sh rewrites bash scripts/ci-test.sh to agent-test.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"bash scripts/ci-test.sh"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-test.sh"* ]]
}

@test "agent-command-router.sh rewrites scripts/lint.sh to agent-lint.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"scripts/lint.sh"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-lint.sh"* ]]
}

@test "agent-command-router.sh rewrites cargo clippy to agent-lint.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"cargo clippy"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-lint.sh"* ]]
}

@test "agent-command-router.sh rewrites cargo fmt to agent-fmt.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"cargo fmt"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-fmt.sh"* ]]
}

@test "agent-command-router.sh rewrites prettier --check to agent-fmt.sh" {
  input='{"tool_name":"Bash","tool_input":{"command":"prettier --check ."}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" == *"agent-fmt.sh"* ]]
}

@test "agent-command-router.sh does not rewrite unrelated commands" {
  input='{"tool_name":"Bash","tool_input":{"command":"ls -la"}}'
  result=$(echo "$input" | bash "$HOOK")
  [[ "$result" != *"agent-test.sh"* ]]
  [[ "$result" != *"agent-lint.sh"* ]]
  [[ "$result" != *"agent-fmt.sh"* ]]
}
