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

# CSS build check
echo "→ Checking CSS is in sync with source..."

# Check if styles.css exists in the repo
if [ ! -f "public/styles.css" ]; then
  echo "✗ public/styles.css is missing!"
  echo "  Please run 'npm run build:css' and commit the result."
  exit 1
fi

# Build CSS to a temporary location
TEMP_CSS=$(mktemp)
if ! postcss src/styles.css -o "$TEMP_CSS" > /dev/null 2>&1; then
  echo "✗ CSS build failed!"
  echo "  Please fix CSS build errors."
  rm -f "$TEMP_CSS"
  exit 1
fi

# Compare the built CSS with the committed version
if ! diff -q "$TEMP_CSS" "public/styles.css" > /dev/null 2>&1; then
  echo "✗ CSS is out of sync with source!"
  echo "  The committed public/styles.css does not match the built output."
  echo "  Please run 'npm run build:css' and commit the changes."
  rm -f "$TEMP_CSS"
  exit 1
fi

rm -f "$TEMP_CSS"
echo "✓ CSS is in sync with source"
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
