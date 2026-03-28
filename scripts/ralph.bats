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
