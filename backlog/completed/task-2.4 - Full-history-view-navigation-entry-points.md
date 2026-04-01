---
id: TASK-2.4
title: Full history view + navigation entry points
status: Done
assignee: []
created_date: "2026-03-29 18:03"
updated_date: "2026-03-31 19:58"
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

Implement the full history view at the stub routes created in TASK-2.2 (`/workout/history` and `/workout/history/:exercise_id`). The view contains:

- A toggle at the top: current exercise name | All Exercises
- Default scope: the exercise passed via route param (when accessed from an active session); "All Exercises" (when accessed from the idle Workout tab)
- A reverse-chronological feed grouped by local calendar day, then sub-grouped by exercise within each day
- Multiple exercises on the same calendar day share one date header with separate sub-groups
- Infinite scroll for additional pages
- Day/exercise groups use device local timezone for day boundaries

Add the two entry points that navigate into this view:

- A history icon in the active session header → navigates to `/workout/history/:exercise_id`
- A "View workout history" button on the idle Workout tab → navigates to `/workout/history`

## Acceptance Criteria

<!-- AC:BEGIN -->

- [x] #1 `/workout/history` renders the all-exercises feed by default
- [x] #2 `/workout/history/:exercise_id` renders the feed defaulting to that exercise's tab
- [x] #3 Toggle switches between per-exercise and all-exercises feeds
- [x] #4 Feed is reverse-chronological; days are grouped correctly using device local timezone
- [x] #5 Multiple exercises on the same day appear under one date header with separate sub-groups
- [x] #6 Infinite scroll loads additional pages
- [x] #7 History icon appears in the active session header and navigates correctly
- [x] #8 "View workout history" button appears on the idle Workout tab and navigates to the all-exercises view
- [x] #9 Sets logged during an active session appear in the history feed immediately
- [x] #10 E2E: feed shows correct day groups after logging across multiple exercises/days; toggle switches content; infinite scroll works; correct default scope from each entry point

## Blocked by

- TASK-2.1 (schema + paginated queries)
- TASK-2.2 (router stub routes)

## User stories addressed

- User story 5 (history icon in active workout header)
- User story 6 (exercise / All Exercises toggle)
- User story 7 (defaults to current exercise when accessed from active session)
- User story 8 (all-exercises view is reverse-chronological grouped by day)
- User story 9 (sub-grouped by exercise within each day)
- User story 10 (same-day exercises share one date header)
- User story 15 (View workout history button on idle Workout tab)
- User story 16 (idle tab defaults to All Exercises)
- User story 27 (new sets appear in history immediately)
- User story 28 (day boundaries use device local timezone)
<!-- SECTION:DESCRIPTION:END -->

<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->

Implemented the full history view and its navigation entry points.

Key improvements:

- Updated `HistoryView` in `src/components/history_view.rs` to handle reactive property changes for `exercise_id` using signals.
- Added scope reset logic when `exercise_id` changes (e.g., from a specific exercise to all exercises).
- Ensured the history feed refreshes immediately when a set is logged during an active session (satisfying AC #9) by subscribing to the completed sets count.
- Verified that day grouping correctly uses the device's local timezone.
- Verified infinite scroll functionality.
- Confirmed that navigation from the active session header defaults to the current exercise, while navigation from the idle Workout tab defaults to all exercises.

All E2E tests for the workout history feature are passing.

<!-- SECTION:FINAL_SUMMARY:END -->
