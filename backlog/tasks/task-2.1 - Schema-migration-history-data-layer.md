---
id: TASK-2.1
title: Schema migration + history data layer
status: In Progress
assignee: []
created_date: '2026-03-29 18:02'
updated_date: '2026-03-29 18:07'
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
- [ ] #1 `sessions` table is absent from the schema; DB version is bumped; app detects old schema and replaces it with a fresh one on startup
- [ ] #2 `completed_sets` has `exercise_id INTEGER NOT NULL` (FK to `exercises.id`) and `recorded_at INTEGER NOT NULL` (Unix ms)
- [ ] #3 `log_set` writes `exercise_id` and `recorded_at`; no `session_id` written
- [ ] #4 `start_session` no longer inserts a row anywhere
- [ ] #5 `get_sets_for_exercise` returns correct sets in reverse-chronological order and respects pagination offsets
- [ ] #6 `get_all_sets_paginated` returns sets across exercises in the correct reverse-chronological order
- [ ] #7 `update_set` persists changes; subsequent reads reflect the update
- [ ] #8 `delete_set` removes the set; subsequent reads no longer include it
- [ ] #9 All four queries have DB-layer unit tests (following the pattern in `src/state/db_tests.rs`)

## Blocked by

None — can start immediately

## User stories addressed

Foundational — unblocks all user stories in the PRD
<!-- SECTION:DESCRIPTION:END -->
<!-- AC:END -->
