#!/usr/bin/env bash
# Minimal-output test runner for LLM agents.
# On success:  prints "Passed" and exits 0.
# On failure:  prints only failing test names / error logs and exits non-zero.
#
# Env vars for testing (set by bats tests only — must not be set in production):
#   _BATS_CARGO_TEST_STUB=pass|fail   — stub cargo test result
#   _BATS_NPM_E2E_STUB=pass|fail|skip — stub npm e2e result

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

CARGO_OUTPUT=$(mktemp)
NPM_OUTPUT=$(mktemp)
trap 'rm -f "$CARGO_OUTPUT" "$NPM_OUTPUT"' EXIT

CARGO_OK=0
NPM_OK=0

# ---------------------------------------------------------------------------
# Run cargo tests (forwarding any arguments from the agent command)
# ---------------------------------------------------------------------------
if [ "${_BATS_CARGO_TEST_STUB:-}" = "pass" ]; then
  : # pass
elif [ "${_BATS_CARGO_TEST_STUB:-}" = "fail" ]; then
  printf 'failures:\n    tests::some_test\n\ntest result: FAILED. 0 passed; 1 failed\n' >"$CARGO_OUTPUT"
  CARGO_OK=1
else
  # Real invocation — forward any extra arguments (e.g. test name filters)
  if ! cargo test "$@" >"$CARGO_OUTPUT" 2>&1; then
    CARGO_OK=1
  fi
fi

# ---------------------------------------------------------------------------
# Run npm e2e tests — skipped if cargo tests already failed
# ---------------------------------------------------------------------------
if [ "$CARGO_OK" -ne 0 ]; then
  : # skip e2e when cargo already failed
elif [ "${_BATS_NPM_E2E_STUB:-}" = "skip" ]; then
  : # skip
elif [ "${_BATS_NPM_E2E_STUB:-}" = "pass" ]; then
  : # pass
elif [ "${_BATS_NPM_E2E_STUB:-}" = "fail" ]; then
  printf '  1) Scenario: some e2e scenario\n     Error: expected element to be visible\n' >"$NPM_OUTPUT"
  NPM_OK=1
else
  # Real invocation — start dev server, wait for readiness, then run e2e
  devenv processes up -d test-serve 2>/dev/null || true
  timeout 60 bash -c 'until curl -s http://localhost:3000 > /dev/null; do sleep 1; done' 2>/dev/null || true
  sleep 5
  if ! npm run test:e2e >"$NPM_OUTPUT" 2>&1; then
    NPM_OK=1
  fi
fi

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------
if [ "$CARGO_OK" -eq 0 ] && [ "$NPM_OK" -eq 0 ]; then
  echo "Passed"
  exit 0
fi

# Print only failure-relevant lines.
# The awk block extracts the failures: section for normal test failures.
# If that section is absent (e.g. compile error or panic), fall back to
# grep for error/panic lines, or tail the raw output as a last resort.
if [ "$CARGO_OK" -ne 0 ] && [ -s "$CARGO_OUTPUT" ]; then
  FAILURES=$(awk '/^failures:/{found=1} found{print}' "$CARGO_OUTPUT" | head -40)
  if [ -n "$FAILURES" ]; then
    printf '%s\n' "$FAILURES"
  else
    grep -E '^error|panicked|FAILED' "$CARGO_OUTPUT" | head -40 || \
      tail -20 "$CARGO_OUTPUT"
  fi
fi

if [ "$NPM_OK" -ne 0 ] && [ -s "$NPM_OUTPUT" ]; then
  grep -E 'FAILED|Error:|failing|✘|×|not ok|Scenario.*failed' "$NPM_OUTPUT" | head -40 || \
    head -20 "$NPM_OUTPUT"
fi

exit 1
