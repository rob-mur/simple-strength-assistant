#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TDD_SKILL="${SCRIPT_DIR}/../.claude/skills/tdd/SKILL.md"

if [ $# -lt 1 ]; then
  echo "Usage: ralph <parent-task-id>" >&2
  exit 1
fi

PARENT_ID="$1"
BRANCH="task-${PARENT_ID}"
WORKTREE="worktrees/${BRANCH}"

git worktree add -b "$BRANCH" "$WORKTREE"

SUBTASK_ID=$(backlog task list --parent "$PARENT_ID" --status "To Do" --plain | awk '/^\s+[A-Z]/{print $1; exit}')

if [ -z "$SUBTASK_ID" ]; then
  echo "No To Do subtasks found for task-${PARENT_ID}" >&2
  exit 1
fi

SUBTASK_CONTENT=$(backlog task "$SUBTASK_ID")
TDD_CONTENT=""
if [ -f "$TDD_SKILL" ]; then
  TDD_CONTENT=$(cat "$TDD_SKILL")
fi
PROMPT="${TDD_CONTENT}

---

${SUBTASK_CONTENT}"

devcontainer up --workspace-folder "$WORKTREE"

cleanup() {
  devcontainer down --workspace-folder "$WORKTREE" 2>/dev/null || true
}
trap cleanup EXIT

MAX_RETRIES=3
ATTEMPT=0
ERROR_CONTEXT=""

while [ $ATTEMPT -lt $MAX_RETRIES ]; do
  ATTEMPT=$((ATTEMPT + 1))

  if [ -n "$ERROR_CONTEXT" ]; then
    # Only the most recent failure is included; earlier errors are dropped to keep prompt size bounded.
    # Cap at 8 KB to avoid hitting API context limits on verbose build/test output.
    TRUNCATED_CONTEXT=$(printf '%s' "$ERROR_CONTEXT" | tail -c 8000)
    FULL_PROMPT="${PROMPT}

---

Attempt ${ATTEMPT} of ${MAX_RETRIES} failed with the following output:
${TRUNCATED_CONTEXT}"
  else
    FULL_PROMPT="$PROMPT"
  fi

  if CLAUDE_OUT=$(devcontainer exec --workspace-folder "$WORKTREE" \
      -e CLAUDE_CODE_OAUTH_TOKEN="${CLAUDE_CODE_OAUTH_TOKEN:-}" \
      -- devenv shell -- claude --print --dangerously-skip-permissions -p "$FULL_PROMPT" 2>&1); then

    if TEST_OUT=$(devcontainer exec --workspace-folder "$WORKTREE" -- devenv shell -- devenv test 2>&1); then
      backlog task edit "$SUBTASK_ID" --status "Done"
      exit 0
    else
      ERROR_CONTEXT="${CLAUDE_OUT}
${TEST_OUT}"
    fi
  else
    ERROR_CONTEXT="$CLAUDE_OUT"
  fi
done

backlog task edit "$SUBTASK_ID" --status "Blocked"
exit 1
