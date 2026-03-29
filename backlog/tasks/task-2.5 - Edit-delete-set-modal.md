---
id: TASK-2.5
title: Edit/delete set modal
status: To Do
assignee: []
created_date: '2026-03-29 18:03'
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

Make every set row in the history feed (from TASK-2.4) tappable. Tapping opens an edit modal that reuses the existing `TapeMeasure` and `RPESlider` components, pre-populated with the set's current values. The modal fetches the exercise's `min_weight` and `increment` via the set's `exercise_id` to configure TapeMeasure constraints. It contains:

- A Save button — calls `update_set` and refreshes the feed row in place
- A Delete button — calls `delete_set` and removes the row; if the row was the last in its day/exercise group, the group disappears automatically

## Acceptance criteria

- [ ] Tapping a set row in the history view opens the edit modal
- [ ] Modal is pre-populated with the set's current weight, reps, and RPE
- [ ] TapeMeasure is configured with the correct `min_weight` and `increment` for that exercise
- [ ] Save persists the edited values; the feed reflects the update without a full reload
- [ ] Delete removes the set from the feed; the row disappears immediately
- [ ] Deleting the last set in a day/exercise group removes that group from the feed
- [ ] E2E: edit weight/reps/RPE and save → updated values appear in feed; delete set → row removed; delete last set in group → group disappears

## Blocked by

- TASK-2.1 (schema + `update_set` / `delete_set` queries)
- TASK-2.4 (history view with tappable set rows)

## User stories addressed

- User story 11 (tap set row to open edit modal)
- User story 12 (edit modal uses TapeMeasure + RPESlider)
- User story 13 (Delete button in edit modal)
- User story 14 (empty day/exercise groups disappear automatically)
<!-- SECTION:DESCRIPTION:END -->
