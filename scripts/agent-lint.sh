#!/usr/bin/env bash
# Minimal-output lint runner for LLM agents.
# On success:  prints "Passed" and exits 0.
# On failure:  prints only offending file paths and error messages, exits non-zero.
#
# Env vars for testing (set by bats tests only — must not be set in production):
#   _BATS_LINT_STUB=pass|fail — stub lint result

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

LINT_OUTPUT=$(mktemp)
trap 'rm -f "$LINT_OUTPUT"' EXIT

LINT_OK=0

if [ "${_BATS_LINT_STUB:-}" = "pass" ]; then
  : # pass
elif [ "${_BATS_LINT_STUB:-}" = "fail" ]; then
  printf 'error[E0502]: some lint error\n --> src/main.rs:10:5\n  |\n10 |     bad_code();\n' >"$LINT_OUTPUT"
  LINT_OK=1
else
  # Real invocation — mirrors lint.sh but captures all output
  if ! bash "$SCRIPT_DIR/lint.sh" >"$LINT_OUTPUT" 2>&1; then
    LINT_OK=1
  fi
fi

if [ "$LINT_OK" -eq 0 ]; then
  echo "Passed"
  exit 0
fi

# Print only actionable failure lines (file paths, error codes, error messages)
if [ -s "$LINT_OUTPUT" ]; then
  grep -E \
    '^error|^warning.*error|warning\[|error\[| --> |✗|FAILED|\bfailed\b|Diff|differs|not formatted' \
    "$LINT_OUTPUT" | head -60 || head -30 "$LINT_OUTPUT"
fi

exit 1
