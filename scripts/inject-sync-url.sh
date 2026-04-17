#!/usr/bin/env bash
# inject-sync-url.sh — replace the %%SYNC_BASE_URL%% placeholder in the built
# index.html with the value of the SYNC_BASE_URL environment variable.
#
# Usage:
#   SYNC_BASE_URL=https://sync.example.com ./scripts/inject-sync-url.sh dist/public/index.html
#
# If SYNC_BASE_URL is not set, the placeholder is removed so that the Wasm
# fallback ("/api") takes effect.

set -euo pipefail

INDEX_FILE="${1:-dist/public/index.html}"
URL="${SYNC_BASE_URL:-}"

if [ ! -f "$INDEX_FILE" ]; then
  echo "Error: $INDEX_FILE not found" >&2
  exit 1
fi

if [ -z "$URL" ]; then
  echo "Warning: SYNC_BASE_URL not set — sync will fall back to /api" >&2
fi

# Use | as sed delimiter since URLs contain slashes
sed -i "s|%%SYNC_BASE_URL%%|${URL}|g" "$INDEX_FILE"

echo "Injected SYNC_BASE_URL='${URL}' into $INDEX_FILE"
