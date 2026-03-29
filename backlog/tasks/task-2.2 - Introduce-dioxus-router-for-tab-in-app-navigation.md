---
id: TASK-2.2
title: Introduce dioxus-router for tab + in-app navigation
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

Replace the current signal-based tab switching with `dioxus-router`. Define the top-level routes (`/workout`, `/library`) and wire the tab bar to use router navigation instead of directly mutating a signal. Implement the full navigation contract specified in the PRD:

- The browser/OS back button pops the router stack across tab boundaries
- Each tab maintains its own nested navigation history; switching tabs restores the last location within that tab
- Tapping the active tab in the tab bar navigates to that tab's root route
- The tab bar remains visible from all routes

Stub out the nested routes (`/workout/history`, `/workout/history/:exercise_id`, `/library/:exercise_id`) as empty placeholders so downstream slices can fill them in.

## Acceptance criteria

- [ ] `dioxus-router` is added as a dependency; signal-based tab switching is removed
- [ ] Tab bar navigates via router; `/workout` and `/library` are the root routes
- [ ] Browser back button navigates back within the app
- [ ] Switching to Library and back restores the previous location within the Workout tab (and vice versa)
- [ ] Tapping the active tab returns to that tab's root route
- [ ] Tab bar is visible from all routes
- [ ] Stub routes exist for `/workout/history`, `/workout/history/:exercise_id`, `/library/:exercise_id`
- [ ] E2E tests: tab switching works; back gesture navigates back; tab-state is preserved on tab switch; tapping active tab goes to root

## Blocked by

None — can be developed in parallel with TASK-2.1

## User stories addressed

- User story 22 (back button in all history/detail views)
- User story 23 (system back button)
- User story 24 (tab bar always visible)
- User story 25 (tapping active tab returns to root)
- User story 26 (tab remembers navigation state)
<!-- SECTION:DESCRIPTION:END -->
