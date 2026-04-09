#!/usr/bin/env bash
# Minimal-output format checker for LLM agents.
# On success:  prints "Passed" and exits 0.
# On failure:  lists only unformatted files and exits non-zero.
#
# Env vars for testing (set by bats tests only — must not be set in production):
#   _BATS_FMT_STUB=pass|fail — stub format check result

set -uo pipefail

FMT_OUTPUT=$(mktemp)
CARGO_FMT_OUT=$(mktemp)
PRETTIER_OUT=$(mktemp)
trap 'rm -f "$FMT_OUTPUT" "$CARGO_FMT_OUT" "$PRETTIER_OUT"' EXIT

FMT_OK=0

if [ "${_BATS_FMT_STUB:-}" = "pass" ]; then
  : # pass
elif [ "${_BATS_FMT_STUB:-}" = "fail" ]; then
  printf 'src/main.rs\nsrc/components/exercise.rs\n' >"$FMT_OUTPUT"
  FMT_OK=1
else
  # Real invocation: check cargo fmt and prettier formatting
  if ! cargo fmt -- --check >"$CARGO_FMT_OUT" 2>&1; then
    echo "Run \`cargo fmt\` to fix formatting" >> "$FMT_OUTPUT"
    FMT_OK=1
  fi

  if ! prettier --check . >"$PRETTIER_OUT" 2>&1; then
    grep -oE '\S+\.(ts|tsx|js|jsx|json|css|md)' "$PRETTIER_OUT" >> "$FMT_OUTPUT" || true
    FMT_OK=1
  fi
fi

if [ "$FMT_OK" -eq 0 ]; then
  echo "Passed"
  exit 0
fi

# Print only unformatted file paths
if [ -s "$FMT_OUTPUT" ]; then
  sort -u "$FMT_OUTPUT"
fi

exit 1
