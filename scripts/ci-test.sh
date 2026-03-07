#!/usr/bin/env bash
set -e

devenv processes up -d test-serve

cargo test

echo "Waiting for http://localhost:8080 to be ready..."
timeout 60 bash -c 'until curl -s http://localhost:8080 > /dev/null; do sleep 1; done'

echo "Waiting for WASM bundle to be ready..."
sleep 5

if ! curl -s http://localhost:8080/assets/*.wasm > /dev/null 2>&1; then
  echo "Warning: WASM bundle may not be fully ready, but proceeding with tests..."
fi

npm run test:e2e
