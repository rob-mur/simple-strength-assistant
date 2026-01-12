#!/usr/bin/env bash
set -e

echo "Running linting checks..."
echo ""

# Commit message validation
echo "→ Validating commit messages..."
if [ -n "$GITHUB_ACTIONS" ]; then
  # In CI, check commits in the PR
  npx commitlint --from "$BASE_SHA" --to "$HEAD_SHA" --verbose
else
  # Locally, check the last commit
  npx commitlint --from HEAD~1 --to HEAD --verbose
fi
echo "✓ Commit messages valid"
echo ""

# Format check
echo "→ Checking code formatting..."
cargo fmt -- --check
echo "✓ Code formatting valid"
echo ""

# Clippy linting
echo "→ Running clippy..."
cargo clippy -- -D warnings
echo "✓ Clippy checks passed"
echo ""

echo "All linting checks passed! ✓"
