#!/usr/bin/env bash
# Minimal-output format runner for LLM agents.
# Applies formatting (cargo fmt, prettier --write) and reports which files changed.
# On success (no files needed formatting):  prints "Passed" and exits 0.
# On fix:  lists formatted files and exits 0 (mutations applied).
#
# Any extra arguments are forwarded to cargo fmt.
#
# Env vars for testing (set by bats tests only — must not be set in production):
#   _BATS_FMT_STUB=pass|fail — stub format check result

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

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
  # Check which files need formatting, then apply fixes.
  # cargo fmt: check first, apply if needed, report changed files.
  if ! cargo fmt -- --check >"$CARGO_FMT_OUT" 2>&1; then
    # Extract file paths from the --check output (lines like "Diff in /path/to/file.rs")
    grep -oE 'Diff in \S+' "$CARGO_FMT_OUT" | sed 's/^Diff in //' >> "$FMT_OUTPUT" || true
    # Apply the formatting
    cargo fmt "$@" 2>/dev/null || true
    FMT_OK=1
  fi

  if ! prettier --check . >"$PRETTIER_OUT" 2>&1; then
    # Prettier emits [warn] lines for unformatted files; parse those directly
    grep '^\[warn\]' "$PRETTIER_OUT" | sed 's/^\[warn\] //' >> "$FMT_OUTPUT" || true
    # Apply the formatting
    prettier --write . >/dev/null 2>&1 || true
    FMT_OK=1
  fi
fi

if [ "$FMT_OK" -eq 0 ]; then
  echo "Passed"
  exit 0
fi

# Report which files were formatted
if [ -s "$FMT_OUTPUT" ]; then
  echo "Formatted:"
  sort -u "$FMT_OUTPUT"
fi

exit 0
