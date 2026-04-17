#!/usr/bin/env bats
# Tests for scripts/inject-sync-url.sh
# Verifies that the SYNC_BASE_URL placeholder injection works correctly for
# all edge cases: set, unset, empty, trailing slashes, and un-replaced tokens.

SCRIPTS_DIR="$(cd "$(dirname "$BATS_TEST_FILENAME")/../scripts" && pwd)"

setup() {
  TMPDIR="$(mktemp -d)"
  # Create a minimal index.html with the placeholder
  cat > "$TMPDIR/index.html" <<'HTML'
<script>
  window.SYNC_BASE_URL = "%%SYNC_BASE_URL%%";
</script>
HTML
}

teardown() {
  rm -rf "$TMPDIR"
}

# ---------------------------------------------------------------------------
# Happy path: SYNC_BASE_URL is set
# ---------------------------------------------------------------------------

@test "replaces placeholder with the provided URL" {
  SYNC_BASE_URL="https://sync.clarob.uk/api" run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/index.html"
  [ "$status" -eq 0 ]
  run cat "$TMPDIR/index.html"
  [[ "$output" == *'window.SYNC_BASE_URL = "https://sync.clarob.uk/api"'* ]]
  [[ "$output" != *'%%SYNC_BASE_URL%%'* ]]
}

@test "outputs confirmation message with the injected URL" {
  SYNC_BASE_URL="https://sync.clarob.uk/api" run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/index.html"
  [ "$status" -eq 0 ]
  [[ "$output" == *"Injected SYNC_BASE_URL='https://sync.clarob.uk/api'"* ]]
}

# ---------------------------------------------------------------------------
# Fallback: SYNC_BASE_URL is unset or empty
# ---------------------------------------------------------------------------

@test "removes placeholder when SYNC_BASE_URL is unset" {
  unset SYNC_BASE_URL
  run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/index.html"
  [ "$status" -eq 0 ]
  run cat "$TMPDIR/index.html"
  [[ "$output" == *'window.SYNC_BASE_URL = ""'* ]]
  [[ "$output" != *'%%SYNC_BASE_URL%%'* ]]
}

@test "prints warning when SYNC_BASE_URL is unset" {
  unset SYNC_BASE_URL
  run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/index.html"
  [ "$status" -eq 0 ]
  [[ "$output" == *"Warning: SYNC_BASE_URL not set"* ]]
}

@test "removes placeholder when SYNC_BASE_URL is empty string" {
  SYNC_BASE_URL="" run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/index.html"
  [ "$status" -eq 0 ]
  run cat "$TMPDIR/index.html"
  [[ "$output" == *'window.SYNC_BASE_URL = ""'* ]]
}

# ---------------------------------------------------------------------------
# Error: file not found
# ---------------------------------------------------------------------------

@test "exits with error when target file does not exist" {
  run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/nonexistent.html"
  [ "$status" -ne 0 ]
  [[ "$output" == *"not found"* ]]
}

# ---------------------------------------------------------------------------
# Edge case: URL with trailing slash
# ---------------------------------------------------------------------------

@test "preserves trailing slash in URL (build_url trims it at runtime)" {
  SYNC_BASE_URL="https://sync.clarob.uk/api/" run bash "$SCRIPTS_DIR/inject-sync-url.sh" "$TMPDIR/index.html"
  [ "$status" -eq 0 ]
  run cat "$TMPDIR/index.html"
  [[ "$output" == *'window.SYNC_BASE_URL = "https://sync.clarob.uk/api/"'* ]]
}
