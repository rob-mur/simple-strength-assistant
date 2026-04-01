---
id: TASK-2
title: >-
  feat: workout history — view past sessions per-exercise and across all
  exercises
status: To Do
assignee: []
created_date: "2026-03-29 18:02"
labels:
  - feature
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->

Parent task mirroring GH #51.

## Summary

Enrich the Workout tab with collapsible per-exercise history while logging, introduce a full history view (accessible from the Workout tab and Library), allow editing/deleting individual sets, and add an exercise detail view in the Library. Underpinned by a schema change that drops the `sessions` table and derives sessions at query time from `completed_sets` grouped by exercise + local calendar day.

## GitHub Issue

#51

## Out of Scope

- Exercise rename UI
- Statistics/analytics (1RM, charts)
- Bulk import/export
- Filtering/searching history
- Cross-device sync
- Data migration (clean break)
<!-- SECTION:DESCRIPTION:END -->
