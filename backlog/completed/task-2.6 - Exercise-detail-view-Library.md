---
id: TASK-2.6
title: Exercise detail view (Library)
status: Done
assignee: []
created_date: "2026-03-29 18:04"
updated_date: "2026-04-01 08:15"
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

Make exercise cards in the Library navigable. Tapping a card navigates to `/library/:exercise_id` (stub created in TASK-2.2). The detail view has:

- **Header**: back chevron, exercise name, Edit button, Start button
- **Body**: the same paginated per-exercise history feed component built in TASK-2.4, scoped to this exercise

Button behaviour:

- **Start** → calls `start_session` for this exercise and navigates to `/workout`
- **Edit** → opens the existing exercise metadata form (same form used from the Library list)
- **Back chevron** → navigates back to `/library`

## Acceptance Criteria

<!-- AC:BEGIN -->

- [x] #1 Tapping an exercise card in the Library navigates to `/library/:exercise_id`
- [x] #2 Header shows back chevron, exercise name, Edit button, and Start button
- [x] #3 Body shows the full paginated per-exercise history feed for that exercise (reusing the component from TASK-2.4)
- [x] #4 Start begins a session for the exercise and navigates to `/workout`
- [x] #5 Edit opens the exercise metadata form; saving returns to the detail view with updated name
- [x] #6 Back chevron returns to the Library list
- [x] #7 E2E: open detail view from library; history feed shows correct sets; Start switches to workout with session active; Edit updates exercise and returns to detail view; back returns to library

## Blocked by

- TASK-2.1 (schema + `get_sets_for_exercise` query)
- TASK-2.2 (router with `/library/:exercise_id` stub)
- TASK-2.4 (per-exercise history feed component to reuse)

## User stories addressed

- User story 17 (tap exercise card → detail view)
- User story 18 (detail view header with back, name, Edit, Start)
- User story 19 (full paginated history in detail view)
- User story 20 (Start begins session + switches to Workout tab)
- User story 21 (Edit opens exercise metadata form)
  <!-- SECTION:DESCRIPTION:END -->
  <!-- AC:END -->
