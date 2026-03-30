#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TDD_SKILL="${SCRIPT_DIR}/../.claude/skills/tdd/SKILL.md"

AI_ENGINE="claude"
PARENT_ID=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --ai)
      AI_ENGINE="$2"
      shift 2
      ;;
    *)
      if [ -z "$PARENT_ID" ]; then
        PARENT_ID="$1"
      else
        echo "Unknown argument: $1" >&2
        exit 1
      fi
      shift
      ;;
  esac
done

if [ -z "$PARENT_ID" ]; then
  echo "Usage: ralph [--ai <claude|gemini>] <parent-task-id>" >&2
  exit 1
fi
BRANCH="task-${PARENT_ID}"
WORKTREE="worktrees/${BRANCH}"

if [ -d "$WORKTREE" ]; then
  echo "Worktree $WORKTREE already exists, reusing it"
elif git show-ref --quiet --verify "refs/heads/$BRANCH"; then
  git worktree add "$WORKTREE" "$BRANCH"
else
  git worktree add -b "$BRANCH" "$WORKTREE"
fi

# Ensure directory exists for realpath
mkdir -p "$WORKTREE"
WORKTREE_ABS=$(realpath "$WORKTREE")
MAIN_REPO_ROOT=$(git rev-parse --show-toplevel)
LOG_FILE="$WORKTREE_ABS/.ralph.log"

SUBTASK_IDS=$(backlog task list --parent "$PARENT_ID" --status "To Do" --plain | awk '/^\s+[A-Z]/{print $1}')

if [ -z "$SUBTASK_IDS" ]; then
  echo "No To Do subtasks found for task-${PARENT_ID}" >&2
  exit 1
fi

# One container is shared for the entire run; all subtasks execute inside the same
# worktree state, so changes made by an earlier subtask are visible to later ones.
# We mount the main repository root at the same path inside the container to ensure 
# absolute gitdir paths in worktrees resolve correctly for git-hooks.
devcontainer up --workspace-folder "$WORKTREE" \
  --mount "type=bind,source=$MAIN_REPO_ROOT,target=$MAIN_REPO_ROOT"

if [ "$AI_ENGINE" = "gemini" ]; then
  # Sync Gemini credentials to the devcontainer
  # Find the container ID based on the workspace folder label
  WORKTREE_ABS=$(realpath "$WORKTREE")
  CONTAINER_ID=$(docker ps -q --filter "label=devcontainer.local_folder=$WORKTREE_ABS")
  if [ -n "$CONTAINER_ID" ]; then
    # We use devcontainer exec to create the directory as the correct user
    devcontainer exec --workspace-folder "$WORKTREE" mkdir -p /home/vscode/.gemini
    [ -f "$HOME/.gemini/oauth_creds.json" ] && docker cp "$HOME/.gemini/oauth_creds.json" "$CONTAINER_ID:/home/vscode/.gemini/oauth_creds.json"
    [ -f "$HOME/.gemini/gemini-credentials.json" ] && docker cp "$HOME/.gemini/gemini-credentials.json" "$CONTAINER_ID:/home/vscode/.gemini/gemini-credentials.json"
    [ -f "$HOME/.gemini/settings.json" ] && docker cp "$HOME/.gemini/settings.json" "$CONTAINER_ID:/home/vscode/.gemini/settings.json"
  fi
fi

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
  PROMPT="You are an autonomous coding agent with no user present. Do not ask for confirmation or approval at any stage — proceed directly to implementation. The Acceptance Criteria in the task are your Definition of Done; ignore any message saying \"No Definition of Done items defined\".

---

${TDD_CONTENT}

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

    echo ""
    echo "[ralph] ${SUBTASK_ID}: running ${AI_ENGINE} (attempt ${ATTEMPT}/${MAX_RETRIES})..."

    PROMPT_FILE="$(mktemp "$WORKTREE_ABS/.ralph_prompt_XXXXXX")"
    printf '%s' "$FULL_PROMPT" > "$PROMPT_FILE"

    AI_TMP="$(mktemp)"
    if [ "$AI_ENGINE" = "claude" ]; then
      AI_EXIT=0
      devcontainer exec --workspace-folder "$WORKTREE" \
          --remote-env CLAUDE_CODE_OAUTH_TOKEN="${CLAUDE_CODE_OAUTH_TOKEN:-}" \
          -- devenv shell claude --output-format stream-json --verbose --dangerously-skip-permissions -p "$FULL_PROMPT" 2>&1 \
          | tee -a "$LOG_FILE" | tee "$AI_TMP" || AI_EXIT=$?
    else
      # For gemini, we use a prompt file redirected to stdin to handle large prompts correctly
      AI_EXIT=0
      devcontainer exec --workspace-folder "$WORKTREE" \
          --remote-env GEMINI_FORCE_FILE_STORAGE=true \
          --remote-env GOOGLE_CLOUD_PROJECT="${GOOGLE_CLOUD_PROJECT:-}" \
          -- bash -c "devenv shell gemini --yolo  --prompt '' < '/workspaces/$(basename "$WORKTREE_ABS")/$(basename "$PROMPT_FILE")'" 2>&1 \
          | tee -a "$LOG_FILE" | tee "$AI_TMP" || AI_EXIT=$?
    fi
    AI_OUT=$(cat "$AI_TMP"); rm -f "$AI_TMP"
    rm -f "$PROMPT_FILE"

    if [ $AI_EXIT -eq 0 ]; then
      echo ""
      echo "[ralph] ${SUBTASK_ID}: running tests..."
      TEST_TMP="$(mktemp)"
      TEST_EXIT=0
      devcontainer exec --workspace-folder "$WORKTREE" -- devenv shell devenv test 2>&1 \
          | tee -a "$LOG_FILE" | tee "$TEST_TMP" || TEST_EXIT=$?
      TEST_OUT=$(cat "$TEST_TMP"); rm -f "$TEST_TMP"
      if [ $TEST_EXIT -eq 0 ]; then
        backlog task edit "$SUBTASK_ID" --status "Done"
        ERROR_CONTEXT=""
        break
      else
        ERROR_CONTEXT="--- AI output ($AI_ENGINE) ---
${AI_OUT}
--- Test output ---
${TEST_OUT}"
      fi
    else
      ERROR_CONTEXT="--- AI output ($AI_ENGINE) ---
${AI_OUT}"
    fi
  done

  if [ $ATTEMPT -eq $MAX_RETRIES ] && [ -n "$ERROR_CONTEXT" ]; then
    echo "Subtask $SUBTASK_ID failed after $MAX_RETRIES attempts:" >&2
    echo "$ERROR_CONTEXT" >&2
    backlog task edit "$SUBTASK_ID" --status "To Do"
    exit 1
  fi
done <<< "$SUBTASK_IDS"

git push origin "$BRANCH"
gh pr create --base main --title "task-${PARENT_ID}" --body "Closes task-${PARENT_ID}"
