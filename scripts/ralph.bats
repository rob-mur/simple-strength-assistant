#!/usr/bin/env bats

SCRIPT="$BATS_TEST_DIRNAME/ralph.sh"

setup() {
  # Temp dir acts as our fake repo root
  export TMPDIR_ROOT
  TMPDIR_ROOT="$(mktemp -d)"

  # Stub bin directory — all external commands are mocked here
  export STUB_BIN="$TMPDIR_ROOT/bin"
  mkdir -p "$STUB_BIN"
  export PATH="$STUB_BIN:$PATH"

  # Make a real git repo so worktree commands have something to work with
  export REPO_DIR="$TMPDIR_ROOT/repo"
  mkdir -p "$REPO_DIR"
  git -C "$REPO_DIR" init -q
  git -C "$REPO_DIR" config user.email "test@test.com"
  git -C "$REPO_DIR" config user.name "Test"
  git -C "$REPO_DIR" commit --allow-empty -q -m "init"
}

teardown() {
  rm -rf "$TMPDIR_ROOT"
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

make_stub() {
  local name="$1"
  local body="$2"
  printf '#!/usr/bin/env bash\n%s\n' "$body" > "$STUB_BIN/$name"
  chmod +x "$STUB_BIN/$name"
}

# ---------------------------------------------------------------------------
# Cycle 1: argument validation
# ---------------------------------------------------------------------------

@test "exits non-zero and prints usage when invoked with no arguments" {
  run bash "$SCRIPT"
  [ "$status" -ne 0 ]
  [[ "$output" == *"Usage"* ]]
}

# ---------------------------------------------------------------------------
# Cycle 2: git worktree creation
# ---------------------------------------------------------------------------

@test "creates worktree at worktrees/task-<id> on branch task-<id>" {
  # Stub out everything that comes after worktree creation
  make_stub backlog 'echo ""; exit 1'   # no subtasks → script will exit after worktree
  make_stub devcontainer 'exit 0'
  make_stub claude 'exit 0'
  make_stub devenv 'exit 0'

  # Run from inside the real git repo; allow non-zero (no subtasks found)
  cd "$REPO_DIR"
  run bash "$SCRIPT" 42
  # The worktree directory must exist
  [ -d "worktrees/task-42" ]
  # The branch must exist
  git branch --list task-42 | grep -q task-42
}

# ---------------------------------------------------------------------------
# Cycle 3: subtask discovery
# ---------------------------------------------------------------------------

@test "exits non-zero with a message when no To Do subtasks found" {
  make_stub backlog 'echo ""'  # empty output → no subtasks
  make_stub devcontainer 'exit 0'

  cd "$REPO_DIR"
  run bash "$SCRIPT" 10
  [ "$status" -ne 0 ]
  [[ "$output" == *"No To Do subtasks"* ]]
}

# ---------------------------------------------------------------------------
# Cycle 4: success path — subtask marked Done
# ---------------------------------------------------------------------------

@test "marks subtask Done when devenv test passes" {
  # Track what backlog was called with
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-5.1 - test task\n"; fi
echo "$@" >> "$TMPDIR_ROOT/backlog.calls"
EOF
  chmod +x "$STUB_BIN/backlog"
  # devcontainer: pass all invocations (up, exec-claude, exec-devenv-test, down)
  make_stub devcontainer 'exit 0'

  cd "$REPO_DIR"
  run bash "$SCRIPT" 5
  [ "$status" -eq 0 ]
  grep -q "task edit TASK-5.1.*--status.*Done" "$TMPDIR_ROOT/backlog.calls"
}

@test "does NOT mark subtask Done when devenv test fails" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-6.1 - test task\n"; fi
echo "$@" >> "$TMPDIR_ROOT/backlog.calls"
EOF
  chmod +x "$STUB_BIN/backlog"
  # devcontainer: fail when it's the devenv test invocation
  make_stub devcontainer '[[ "$*" == *"devenv test"* ]] && exit 1; exit 0'

  cd "$REPO_DIR"
  run bash "$SCRIPT" 6
  [ "$status" -ne 0 ]
  # No "Done" status update should appear
  if [ -f "$TMPDIR_ROOT/backlog.calls" ]; then
    run grep "Done" "$TMPDIR_ROOT/backlog.calls"
    [ "$status" -ne 0 ]
  fi
}

# ---------------------------------------------------------------------------
# Cycle 5: container teardown
# ---------------------------------------------------------------------------

@test "tears down container after successful run" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-7.1 - test task\n"; fi
EOF
  chmod +x "$STUB_BIN/backlog"
  cat > "$STUB_BIN/devcontainer" <<'EOF'
#!/usr/bin/env bash
echo "$@" >> "$TMPDIR_ROOT/devcontainer.calls"
exit 0
EOF
  chmod +x "$STUB_BIN/devcontainer"

  cd "$REPO_DIR"
  run bash "$SCRIPT" 7
  [ "$status" -eq 0 ]
  grep -q "down\|stop" "$TMPDIR_ROOT/devcontainer.calls"
}

@test "tears down container even when devenv test fails" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-8.1 - test task\n"; fi
EOF
  chmod +x "$STUB_BIN/backlog"
  cat > "$STUB_BIN/devcontainer" <<'EOF'
#!/usr/bin/env bash
echo "$@" >> "$TMPDIR_ROOT/devcontainer.calls"
[[ "$*" == *"devenv test"* ]] && exit 1
exit 0
EOF
  chmod +x "$STUB_BIN/devcontainer"

  cd "$REPO_DIR"
  run bash "$SCRIPT" 8
  [ "$status" -ne 0 ]
  grep -q "down\|stop" "$TMPDIR_ROOT/devcontainer.calls"
}

# ---------------------------------------------------------------------------
# Cycle 6: retry logic — 3 failures marks Blocked
# Task IDs 20-24 are reserved for retry-logic tests (cycles 6-10).
# ---------------------------------------------------------------------------

@test "marks subtask Blocked after 3 consecutive devenv test failures" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-20.1 - test task\n"; fi
echo "$@" >> "$TMPDIR_ROOT/backlog.calls"
EOF
  chmod +x "$STUB_BIN/backlog"
  # devcontainer: always fail on devenv test
  cat > "$STUB_BIN/devcontainer" <<'EOF'
#!/usr/bin/env bash
echo "$@" >> "$TMPDIR_ROOT/devcontainer.calls"
[[ "$*" == *"devenv test"* ]] && { echo "test failure output"; exit 1; }
exit 0
EOF
  chmod +x "$STUB_BIN/devcontainer"

  cd "$REPO_DIR"
  run bash "$SCRIPT" 20
  [ "$status" -ne 0 ]
  grep -q "task edit TASK-20.1.*--status.*Blocked" "$TMPDIR_ROOT/backlog.calls"
}

# ---------------------------------------------------------------------------
# Cycle 7: retry loop runs exactly 3 times before giving up
# ---------------------------------------------------------------------------

@test "invokes claude exactly 3 times before marking Blocked" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-21.1 - test task\n"; fi
EOF
  chmod +x "$STUB_BIN/backlog"
  cat > "$STUB_BIN/devcontainer" <<'EOF'
#!/usr/bin/env bash
[[ "$*" == *"devenv test"* ]] && { echo "fail"; exit 1; }
# Detect claude via substring match on the command line passed to devcontainer exec.
# Works because ralph.sh always passes "-- devenv shell -- claude" as a literal sequence.
[[ "$*" == *"claude"* ]] && echo "claude-call" >> "$TMPDIR_ROOT/claude.calls"
exit 0
EOF
  chmod +x "$STUB_BIN/devcontainer"

  cd "$REPO_DIR"
  run bash "$SCRIPT" 21
  [ "$status" -ne 0 ]
  [ "$(wc -l < "$TMPDIR_ROOT/claude.calls")" -eq 3 ]
}

# ---------------------------------------------------------------------------
# Cycle 8: error context fed back to next Claude invocation
# ---------------------------------------------------------------------------

@test "passes previous failure output as context to next Claude invocation" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-22.1 - test task\n"; fi
EOF
  chmod +x "$STUB_BIN/backlog"
  ATTEMPT_FILE="$TMPDIR_ROOT/attempt"
  echo "0" > "$ATTEMPT_FILE"
  cat > "$STUB_BIN/devcontainer" <<'STUB'
#!/usr/bin/env bash
if [[ "$*" == *"devenv test"* ]]; then
  echo "SPECIFIC_ERROR_MARKER"
  exit 1
fi
if [[ "$*" == *"claude"* ]]; then
  # echo "$@" flattens all args onto one line; FULL_PROMPT is a single -p argument so
  # its content (including the error marker) is preserved in the output.
  echo "$@" >> "$TMPDIR_ROOT/claude.invocations"
fi
exit 0
STUB
  chmod +x "$STUB_BIN/devcontainer"

  cd "$REPO_DIR"
  run bash "$SCRIPT" 22
  [ "$status" -ne 0 ]
  # Second (and later) claude invocations should contain the error marker
  # (check lines after the first one)
  tail -n +2 "$TMPDIR_ROOT/claude.invocations" | grep -q "SPECIFIC_ERROR_MARKER"
}

# ---------------------------------------------------------------------------
# Cycle 9: passes on retry → Done, not Blocked
# ---------------------------------------------------------------------------

@test "marks subtask Done when it passes on the second attempt" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-23.1 - test task\n"; fi
echo "$@" >> "$TMPDIR_ROOT/backlog.calls"
EOF
  chmod +x "$STUB_BIN/backlog"
  # Fail devenv test once, then pass.
  # COUNT_FILE lives under TMPDIR_ROOT which is unique per test (setup creates a fresh mktemp dir),
  # so there is no cross-test race for sequential runs. Would need rethinking for parallel bats.
  cat > "$STUB_BIN/devcontainer" <<'STUB'
#!/usr/bin/env bash
if [[ "$*" == *"devenv test"* ]]; then
  COUNT_FILE="$TMPDIR_ROOT/test.count"
  COUNT=$(cat "$COUNT_FILE" 2>/dev/null || echo 0)
  COUNT=$((COUNT + 1))
  echo "$COUNT" > "$COUNT_FILE"
  [ "$COUNT" -gt 1 ] && exit 0
  echo "first attempt failure"; exit 1
fi
exit 0
STUB
  chmod +x "$STUB_BIN/devcontainer"

  cd "$REPO_DIR"
  run bash "$SCRIPT" 23
  [ "$status" -eq 0 ]
  grep -q "task edit TASK-23.1.*--status.*Done" "$TMPDIR_ROOT/backlog.calls"
  # Must NOT be marked Blocked
  run grep "Blocked" "$TMPDIR_ROOT/backlog.calls"
  [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# Cycle 10: loop stops after Blocked (no subsequent subtasks processed)
# ---------------------------------------------------------------------------

@test "exits non-zero immediately after marking subtask Blocked" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-24.1 - test task\n"; fi
echo "$@" >> "$TMPDIR_ROOT/backlog.calls"
EOF
  chmod +x "$STUB_BIN/backlog"
  make_stub devcontainer '[[ "$*" == *"devenv test"* ]] && exit 1; exit 0'

  cd "$REPO_DIR"
  run bash "$SCRIPT" 24
  [ "$status" -ne 0 ]
}

@test "proceeds when a To Do subtask is found" {
  cat > "$STUB_BIN/backlog" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"task list"* ]]; then printf "To Do:\n  TASK-10.1 - test task\n"; fi
EOF
  chmod +x "$STUB_BIN/backlog"
  make_stub devcontainer 'exit 0'
  make_stub claude 'exit 0'
  make_stub devenv 'exit 0'

  cd "$REPO_DIR"
  run bash "$SCRIPT" 10
  [ "$status" -eq 0 ]
}
