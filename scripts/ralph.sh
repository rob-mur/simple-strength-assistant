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

SUBTASK_IDS=$(backlog task list --parent "$PARENT_ID" --status "To Do" --plain | awk '/^\s+[A-Z]/{print $1}')

if [ -z "$SUBTASK_IDS" ]; then
  echo "No To Do subtasks found for task-${PARENT_ID}" >&2
  exit 1
fi

devcontainer up --workspace-folder "$WORKTREE"

cleanup() {
  devcontainer down --workspace-folder "$WORKTREE" 2>/dev/null || true
}
trap cleanup EXIT

TDD_CONTENT=""
if [ -f "$TDD_SKILL" ]; then
  TDD_CONTENT=$(cat "$TDD_SKILL")
fi

while IFS= read -r SUBTASK_ID; do
  [ -z "$SUBTASK_ID" ] && continue

  backlog task edit "$SUBTASK_ID" --status "In Progress"

  SUBTASK_CONTENT=$(backlog task "$SUBTASK_ID")
  PROMPT="${TDD_CONTENT}

---

${SUBTASK_CONTENT}"

  MAX_RETRIES=3
  ATTEMPT=0
  ERROR_CONTEXT=""

  while [ $ATTEMPT -lt $MAX_RETRIES ]; do
    ATTEMPT=$((ATTEMPT + 1))

    if [ -n "$ERROR_CONTEXT" ]; then
      # Only the most recent failure is included; earlier errors are dropped to keep prompt size bounded.
      # Cap at 8 KB to avoid hitting API context limits on verbose build/test output.
      CONTEXT_SIZE=$(printf '%s' "$ERROR_CONTEXT" | wc -c)
      if [ "$CONTEXT_SIZE" -gt 8000 ]; then
        TRUNCATED_CONTEXT="[Output truncated — showing last 8000 of ${CONTEXT_SIZE} bytes]
$(printf '%s' "$ERROR_CONTEXT" | tail -c 8000)"
      else
        TRUNCATED_CONTEXT="$ERROR_CONTEXT"
      fi
      # ATTEMPT has already been incremented for this iteration; the failure being fed back
      # is from the previous attempt, so reference ATTEMPT-1.
      FULL_PROMPT="${PROMPT}

---

Attempt $((ATTEMPT - 1)) of ${MAX_RETRIES} failed with the following output:
${TRUNCATED_CONTEXT}"
    else
      FULL_PROMPT="$PROMPT"
    fi

    if CLAUDE_OUT=$(devcontainer exec --workspace-folder "$WORKTREE" \
        -e CLAUDE_CODE_OAUTH_TOKEN="${CLAUDE_CODE_OAUTH_TOKEN:-}" \
        -- devenv shell -- claude --print --dangerously-skip-permissions -p "$FULL_PROMPT" 2>&1); then

      if TEST_OUT=$(devcontainer exec --workspace-folder "$WORKTREE" -- devenv shell -- devenv test 2>&1); then
        backlog task edit "$SUBTASK_ID" --status "Done"
        break
      else
        ERROR_CONTEXT="--- Claude output ---
${CLAUDE_OUT}
--- Test output ---
${TEST_OUT}"
      fi
    else
      ERROR_CONTEXT="--- Claude output ---
${CLAUDE_OUT}"
    fi
  done

  if [ $ATTEMPT -eq $MAX_RETRIES ] && [ -n "$ERROR_CONTEXT" ]; then
    backlog task edit "$SUBTASK_ID" --status "Blocked"
    exit 1
  fi
done <<< "$SUBTASK_IDS"

git push origin "$BRANCH"
gh pr create --base main --title "task-${PARENT_ID}" --body "Closes task-${PARENT_ID}"
