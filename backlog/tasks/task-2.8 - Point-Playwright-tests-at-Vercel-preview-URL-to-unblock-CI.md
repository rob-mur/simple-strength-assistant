---
id: TASK-2.8
title: Point Playwright tests at Vercel preview URL to unblock CI
status: In Progress
assignee: []
created_date: "2026-04-01 14:08"
updated_date: "2026-04-01 14:14"
labels: []
dependencies: []
parent_task_id: TASK-2
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->

## Goal

Unblock the CI test job by removing the local WASM compilation requirement. Instead of spinning up `dx serve` locally, repoint Playwright's `baseURL` at the Vercel preview deployment that the `build-and-deploy` job already produces.

## Changes Required

1. **CI workflow**: Make the `test` job depend on `build-and-deploy` so the preview URL is available before tests run.
2. **Playwright config**: Replace the `webServer` block with a `baseURL` pointing at the Vercel preview URL (passed in via env var from the workflow).
3. **GitHub Actions**: Pass the preview URL output from the deploy job into the test job as an environment variable.

## Acceptance Criteria

<!-- AC:BEGIN -->

- [ ] #1 CI `test` job no longer compiles WASM locally
- [ ] #2 All existing Playwright scenarios pass against the Vercel preview URL
- [ ] #3 No local `dx serve` process is started during CI
- [ ] #4 Local development still works (falls back to localhost when the env var is absent)
  <!-- SECTION:DESCRIPTION:END -->
  <!-- AC:END -->
