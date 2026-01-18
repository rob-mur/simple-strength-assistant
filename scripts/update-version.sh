#!/usr/bin/env bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Error: Version required as first argument"
  exit 1
fi

echo "Updating version to $VERSION"

# Update Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update package.json
npm version "$VERSION" --no-git-tag-version --allow-same-version

echo "✓ Version updated to $VERSION in Cargo.toml and package.json"
