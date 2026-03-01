#!/usr/bin/env bash
set -e

# Export chromium path for Playwright (from devenv.nix)
# This ensures Playwright uses NixOS-compatible chromium instead of downloaded binaries
# Note: CHROMIUM_EXECUTABLE_PATH must be set by devenv.nix, no fallback needed
if [ -z "$CHROMIUM_EXECUTABLE_PATH" ]; then
  echo "Error: CHROMIUM_EXECUTABLE_PATH not set. Run this script from 'devenv shell'."
  exit 1
fi
export CHROMIUM_EXECUTABLE_PATH

# Run cargo tests
cargo test

# Ensure devenv processes are stopped when the script exits
# Capture exit status before shutting down processes
function cleanup() {
  EXIT_CODE=$?
  devenv processes down
  exit $EXIT_CODE
}
trap cleanup EXIT

# Start test server with test-mode feature enabled (uses in-memory storage instead of OPFS)
devenv processes up -d test-serve

# Wait for the service to be ready
echo "Waiting for http://localhost:8080 to be ready..."
timeout 60 bash -c 'until curl -s http://localhost:8080 > /dev/null; do sleep 1; done'

# Give Dioxus additional time to build and serve the WASM bundle
echo "Waiting for WASM bundle to be ready..."
sleep 5

# Verify WASM bundle is accessible
if ! curl -s http://localhost:8080/assets/*.wasm > /dev/null 2>&1; then
  echo "Warning: WASM bundle may not be fully ready, but proceeding with tests..."
fi

# Run Playwright tests
npm run test:e2e