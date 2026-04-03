---
id: TASK-2.1
title: Schema migration + history data layer
status: Done
assignee: []
created_date: "2026-03-29 18:02"
updated_date: "2026-03-31 18:04"
labels:
  - afk
dependencies: []
parent_task_id: TASK-2
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->

## Parent PRD

TASK-2 / GH #51

## What to build

Drop the `sessions` table entirely and extend `completed_sets` with `exercise_id` and `recorded_at`. Bump the DB version so a fresh schema is applied on first launch after the update. Update `start_session` (no longer creates a DB row) and `log_set` (writes `exercise_id` + `recorded_at` instead of `session_id`). Add the four paginated history queries that all UI slices will call:

- `get_sets_for_exercise(exercise_id, limit, offset)`
- `get_all_sets_paginated(limit, offset)`
- `update_set(set_id, reps, rpe, weight)`
- `delete_set(set_id)`

Day-grouping logic lives in Rust using device local-time conversion (not SQL).

## Acceptance Criteria

<!-- AC:BEGIN -->

- [x] #1 `sessions` table is absent from the schema; DB version is bumped; app detects old schema and replaces it with a fresh one on startup
- [x] #2 `completed_sets` has `exercise_id INTEGER NOT NULL` (FK to `exercises.id`) and `recorded_at INTEGER NOT NULL` (Unix ms)
- [x] #3 `log_set` writes `exercise_id` and `recorded_at`; no `session_id` written
- [x] #4 `start_session` no longer inserts a row anywhere
- [x] #5 `get_sets_for_exercise` returns correct sets in reverse-chronological order and respects pagination offsets
- [x] #6 `get_all_sets_paginated` returns sets across exercises in the correct reverse-chronological order
- [x] #7 `update_set` persists changes; subsequent reads reflect the update
- [x] #8 `delete_set` removes the set; subsequent reads no longer include it
- [x] #9 All four queries have DB-layer unit tests (following the pattern in `src/state/db_tests.rs`)

<!-- AC:END -->

## Blocked by

None — can start immediately

## User stories addressed

Foundational — unblocks all user stories in the PRD

<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->

Verified the database schema migration and history data layer implementation.

- Confirmed sessions table is removed and completed_sets is extended with exercise_id and recorded_at.
- Verified DB version bump and migration logic.
- Confirmed log_set and start_session updates.
- Verified the four paginated history queries: get_sets_for_exercise, get_all_sets_paginated, update_set, and delete_set.
- Fixed code formatting in the task file to pass linting.
- Fixed 2 E2E test failures (Library header visibility and TapeMeasure tick visibility) to ensure a clean test suite.
- All 41 Rust unit tests, 19 Bats functional tests, and 39 Playwright E2E tests are passing.
<!-- SECTION:FINAL_SUMMARY:END -->
