---
id: TASK-2.2
title: Introduce dioxus-router for tab + in-app navigation
status: Done
assignee: []
created_date: "2026-03-29 18:03"
updated_date: "2026-03-31 14:19"
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

## Acceptance Criteria

<!-- AC:BEGIN -->

- [x] #1 `dioxus-router` is added as a dependency; signal-based tab switching is removed
- [x] #2 Tab bar navigates via router; `/workout` and `/library` are the root routes
- [x] #3 Browser back button navigates back within the app
- [x] #4 Switching to Library and back restores the previous location within the Workout tab (and vice versa)
- [x] #5 Tapping the active tab returns to that tab's root route
- [x] #6 Tab bar is visible from all routes
- [x] #7 Stub routes exist for `/workout/history`, `/workout/history/:exercise_id`, `/library/:exercise_id`
- [x] #8 E2E tests: tab switching works; back gesture navigates back; tab-state is preserved on tab switch; tapping active tab goes to root

## Blocked by

None — can be developed in parallel with TASK-2.1

## User stories addressed

- User story 22 (back button in all history/detail views)
- User story 23 (system back button)
- User story 24 (tab bar always visible)
- User story 25 (tapping active tab returns to root)
- User story 26 (tab remembers navigation state)
<!-- SECTION:DESCRIPTION:END -->

<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->

- Replaced signal-based tab switching with `dioxus-router`.
- Implemented `TabNavigationState` at the app root to persist the last-seen route for each tab.
- Configured `Shell` layout to update and restore these routes during tab switching (AC #4).
- Implemented "tap active tab to return to root" logic in `TabBar` on_change (AC #5).
- Added `data-testid` to `TabBar`, `WorkoutView`, and `LibraryView` for robust E2E testing.
- Created `tests/e2e/features/navigation.feature` and corresponding steps in `tests/e2e/steps/navigation.steps.ts` (AC #8).
- Verified browser back button, tapping active tab, and tab state preservation via Playwright.
- Ensured tab bar is visible from all routes and stub routes exist for future slices.
<!-- SECTION:FINAL_SUMMARY:END -->
