# Quick Task 12: please fix the ci-tests (they were working pre linting)

## Task
1. Investigate the cause of Playwright test timeouts (waiting for localhost:8080).
2. Fix compilation errors caused by the `test-mode` feature in `src/state/workout_state.rs`, `src/app.rs`, and `src/state/storage.rs`.
3. Resolve dead code and unused import warnings in `src/state/file_system.rs` and `src/state/mod.rs` when `test-mode` is toggled.
4. Run CI tests again to confirm everything is passing.