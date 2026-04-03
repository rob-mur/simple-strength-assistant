---
id: TASK-2.7
title: Replace resource-intensive Playwright E2E with lighter BDD testing strategy
status: To Do
assignee: []
created_date: "2026-04-01 13:57"
labels: []
dependencies: []
parent_task_id: TASK-2
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->

## Problem

The Playwright E2E tests pass locally but fail in CI (GitHub Actions ubuntu-latest runners: 2 cores, 7 GB RAM). The root cause is that `devenv test` triggers a full Rust → WASM compilation of the Dioxus app via `dx serve --features test-mode` on every CI run, with no build cache. This is extremely resource-intensive and often exceeds available CPU/memory, causing the test server to fail or timeout before Playwright can connect.

## Root Cause Analysis

**Why it fails in CI but not locally:**

1. **No Rust/Cargo build cache**: CI rebuilds the entire WASM bundle from scratch on every run. Locally the incremental compiler cache makes subsequent builds fast.
2. **Double compilation**: Both the `test` job and `build-and-deploy` job independently compile the same WASM — no artifact sharing.
3. **Server startup tolerance is tight**: The Playwright webServer timeout is 300s, but `dx serve` in CI must compile first. On 2 cores this may exceed the window or exhaust memory.
4. **5s WASM sleep guard is bypassed**: `ci-test.sh` includes that guard, but `devenv test` calls `npm run test:e2e` directly, skipping it entirely.

**Why this will not scale:** As the app grows, WASM bundle size and compilation time increase. More E2E scenarios = more browser instances = more memory pressure. The problem compounds without an architectural change.

## Proposed Alternatives (BDD preserved)

All options below keep Gherkin `.feature` files as the authoritative specification.

### Option A — Cargo/WASM build caching + artifact sharing (lowest effort)

Add `Swatinium/rust-cache` to the `test` job and reuse the pre-built dist artifact from `build-and-deploy`. Serve the static bundle with a lightweight HTTP server instead of `dx serve`. No change to test authorship.
**Pros:** Minimal change; fixes immediate CI failures. **Cons:** Still requires a browser.

### Option B — Run Playwright against the Vercel preview URL

Reorder CI so `test` depends on `build-and-deploy`. Point `baseURL` at the Vercel preview. No local WASM compilation in the test job at all.
**Pros:** Test job is fast; the same binary that ships is tested. **Cons:** Adds Vercel as a hard dependency for the test gate.

### Option C — wasm-bindgen-test for component-level BDD

Move feature coverage to `wasm-bindgen-test` tests running inside WASM via `wasm-pack test --headless --chrome`. Step implementations call Dioxus APIs directly rather than driving a browser via CDP.
**Pros:** Much faster per-test; no full-app server needed. **Cons:** Significant rewrite; does not cover routing or cross-component navigation.

### Option D — Hybrid: smoke Playwright + wasm-bindgen-test (recommended long-term)

Reserve 10-15 critical user journeys for Playwright (run against cached/pre-built artifact). Move component-level behaviour (RPE slider, TapeMeasure, step controls) to `wasm-bindgen-test`. Both layers use Gherkin feature files.
**Pros:** Best coverage-to-cost ratio; BDD at both levels. **Cons:** Two test frameworks to maintain.

## Recommended Approach

Start with **Option A** as an immediate fix — low-risk and may be sufficient. Evaluate **Option D** as the long-term architecture as scenario count grows.

## Acceptance Criteria

- CI test job passes reliably without OOM or timeout on ubuntu-latest
- Gherkin `.feature` files remain the source of truth for all user-facing behaviour
- CI test job wall-clock time is reduced relative to the current approach
- No regression in feature coverage for the workout history scenarios
<!-- SECTION:DESCRIPTION:END -->
