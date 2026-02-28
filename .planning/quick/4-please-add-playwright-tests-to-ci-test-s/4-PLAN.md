# Plan for Quick Task 4: Add playwright tests to ci-test script

## Task 1: Update devenv.nix with processes
- Add `processes.serve.exec = "dx serve --port 8080";` to `devenv.nix` to run the app as a background service using devenv.

## Task 2: Remove webServer from playwright config
- Remove the `webServer` block from `playwright.config.ts` so playwright doesn't try to start its own background server, relying on devenv processes instead.

## Task 3: Update ci-test script
- Modify `scripts/ci-test.sh` to:
  - Setup a trap to stop devenv processes on exit (`trap "devenv processes down" EXIT`).
  - Start devenv processes in the background (`devenv processes up -d`).
  - Wait for the server to be ready (e.g., using `curl --retry` or `wait-on`).
  - Run the `cargo test`.
  - Run the `npm run test:e2e` (or `npx playwright test`).
