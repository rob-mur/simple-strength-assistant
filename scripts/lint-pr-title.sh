#!/usr/bin/env bash
# Lints the PR title against commitlint rules.
#
# A squash merge uses the PR title verbatim as the commit message on `main`,
# which feeds semantic-release. A non-conventional title therefore blocks the
# next release. This check runs only when PR_TITLE is in scope (i.e. on a
# pull_request event); otherwise it is a no-op so local runs and push events
# still succeed.

set -e

PR_TITLE="${PR_TITLE:-}"

if [ -z "$PR_TITLE" ]; then
  echo "↷ Skipping PR title lint (no PR_TITLE in scope)"
  exit 0
fi

# Strip Draft/WIP prefixes. Forgejo marks drafts with a "WIP:" title prefix;
# GitLab uses "Draft:". Both are removed on undraft, so the underlying title
# is what actually lands on main and what we need to validate.
STRIPPED_TITLE=$(echo "$PR_TITLE" | sed -E 's/^[[:space:]]*(WIP|Draft)[[:space:]]*:[[:space:]]*//I')

echo "→ Validating PR title..."
if echo "$STRIPPED_TITLE" | npx commitlint --verbose; then
  echo "✓ PR title valid"
else
  echo ""
  echo "✗ PR title does not satisfy commitlint rules:"
  echo "    \"$PR_TITLE\""
  echo "  The PR title becomes the squash-merge commit message on main and feeds"
  echo "  semantic-release. Update it to a conventional-commit subject"
  echo "  (e.g. 'feat(scope): summary')."
  exit 1
fi
