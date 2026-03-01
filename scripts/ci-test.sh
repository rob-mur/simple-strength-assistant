#!/usr/bin/env bash
set -e

# Export chromium path for Playwright (from devenv.nix)
# This ensures Playwright uses NixOS-compatible chromium instead of downloaded binaries
export CHROMIUM_EXECUTABLE_PATH="${CHROMIUM_EXECUTABLE_PATH:-$(which chromium)}"

# Run cargo tests
cargo test

# Ensure devenv processes are stopped when the script exits
trap "devenv processes down" EXIT

# Start necessary background services using devenv processes
devenv processes up -d

# Wait for the service to be ready
echo "Waiting for http://localhost:8080 to be ready..."
timeout 30 bash -c 'until curl -s http://localhost:8080 > /dev/null; do sleep 1; done'

# Run Playwright tests
npm run test:e2e