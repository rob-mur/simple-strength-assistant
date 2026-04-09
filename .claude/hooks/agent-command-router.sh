#!/usr/bin/env bash
# PreToolUse hook: intercepts raw test/lint/format commands and rewrites them
# to the minimal-output agent wrapper scripts.
#
# Claude Code hook protocol:
#   - Input:  JSON on stdin  { "tool_name": "Bash", "tool_input": { "command": "..." } }
#   - Output: JSON on stdout
#     - To rewrite:  { "hookSpecificOutput": { "hookEventName": "PreToolUse",
#                       "permissionDecision": "allow", "updatedInput": { "command": "..." } } }
#     - To pass-through: exit 0 with no output (or empty JSON)
#   - Exit 0 = allow (possibly with rewrite), exit 2 = block

set -uo pipefail

# Read stdin
INPUT=$(cat)

# Only act on Bash tool calls
TOOL_NAME=$(printf '%s' "$INPUT" | jq -r '.tool_name // empty')
if [ "$TOOL_NAME" != "Bash" ]; then
  exit 0
fi

COMMAND=$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty')

# Determine the project root (directory containing .claude/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Match patterns that should be routed to agent wrappers
REWRITE_COMMAND=""

case "$COMMAND" in
  cargo\ test*|"cargo test")
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-test.sh"
    ;;
  "npm run test:e2e"*|"npx playwright"*)
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-test.sh"
    ;;
  "scripts/ci-test.sh"*|"bash scripts/ci-test.sh"*)
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-test.sh"
    ;;
  "scripts/lint.sh"*|"bash scripts/lint.sh"*)
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-lint.sh"
    ;;
  cargo\ clippy*|"cargo clippy")
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-lint.sh"
    ;;
  cargo\ fmt*|"cargo fmt")
    # NOTE: `cargo fmt` (apply) is intentionally converted to a format check via agent-fmt.sh.
    # Agents should not silently mutate source files; agent-fmt.sh reports what needs fixing
    # so the agent can decide whether to apply changes.
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-fmt.sh"
    ;;
  "prettier --check"*|"npx prettier --check"*)
    REWRITE_COMMAND="bash $PROJECT_ROOT/scripts/agent-fmt.sh"
    ;;
  *)
    # No rewrite needed — pass through
    exit 0
    ;;
esac

# Emit rewrite JSON using jq to ensure proper escaping
jq -n \
  --arg cmd "$REWRITE_COMMAND" \
  '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "allow",
      updatedInput: { command: $cmd }
    }
  }'
exit 0
