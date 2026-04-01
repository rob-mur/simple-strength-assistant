---
id: TASK-2.3
title: Collapsible "Previous Sessions" in active workout
status: Done
assignee: []
created_date: "2026-03-29 18:03"
updated_date: "2026-03-30 20:11"
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

Add a collapsible "Previous Sessions" section below the current session's set list in the active workout view. The section uses `get_sets_for_exercise` (added in TASK-2.1), scoped to the currently active exercise. Sets are displayed grouped by local calendar date, with each set on its own row (e.g. "Set 1: 100 kg × 5 @ 7"). The section is collapsed by default and expands on tap. Additional history loads as the user scrolls toward the bottom (infinite scroll).

No routing is required — this is an in-place expansion within the existing active session view.

## Acceptance Criteria

<!-- AC:BEGIN -->

- [x] #1 "Previous Sessions" collapsible appears below the current set list when a session is active
- [x] #2 Section is collapsed by default
- [x] #3 Tapping the section header expands/collapses it
- [x] #4 Expanded section shows sets grouped by date; each set row displays set number, weight, reps, and RPE
- [x] #5 Scrolling to the bottom of the expanded list loads the next page of history (infinite scroll)
- [x] #6 Sets logged in the current session appear in the history feed immediately (reactive update)
- [x] #7 E2E: history section is collapsed by default; log a set, expand section, set appears; scrolling loads more when history is long

## Blocked by

- TASK-2.1 (schema + `get_sets_for_exercise` query)

## User stories addressed

- User story 1 (collapsible Previous Sessions while logging)
- User story 2 (collapsed by default)
- User story 3 (sets grouped by date with per-row display)
- User story 4 (infinite scroll in Previous Sessions)
- User story 27 (new sets appear in history immediately)
<!-- SECTION:DESCRIPTION:END -->

<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->

Completed collapsible "Previous Sessions" in active workout.

- Added a collapsible panel below the current session's set list.
- Implemented grouped history by calendar date using `group_sets_by_day`.
- Fixed reactivity issues where new sets wouldn't appear immediately in the history feed (AC #6) by using signals for props.
- Implemented automatic infinite scroll using `web_sys` `IntersectionObserver` on a sentinel element (AC #5).
- Verified behavior with 4 E2E tests, ensuring that expanding/collapsing works, new sets appear instantly, and scrolling triggers pagination correctly.
- Updated E2E tests to better handle the automatic loading trigger by scrolling elements into view and polling for row count changes.
<!-- SECTION:FINAL_SUMMARY:END -->
