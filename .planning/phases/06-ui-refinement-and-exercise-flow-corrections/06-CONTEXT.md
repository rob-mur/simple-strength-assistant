# Phase 6: UI Refinement and Exercise Flow Corrections Context

## Overview

Phase 6 focuses on polishing the user interface, specifically the navigation bar, and streamlining the workout start flow by centralizing exercise management and session initiation in the Library tab.

## Goals

1.  **Navigation Bar Polish**: Ensure the bottom navigation bar is correctly positioned, styled, and handles mobile safe areas (notches/home indicators).
2.  **Centralized Exercise Management**: Move exercise creation and configuration (min weight, increments) from the "Start Session" flow to the Library tab.
3.  **Streamlined Workout Flow**: Remove the redundant "Start Session" view in the Workout tab. Users will now select an exercise from the Library to start a session.

## Success Criteria

1.  Navigation bar is fixed to the bottom and properly padded for safe areas.
2.  Users can add new exercises directly from the Library tab.
3.  Users can view and edit exercise configurations (name, weighted vs. bodyweight, min weight, increment) in the Library tab.
4.  The Workout tab shows a "No active session" state when idle, with a clear call-to-action to go to the Library.
5.  Starting a workout session is initiated from an exercise's detail view in the Library.

## Current State

- `TabBar` is fixed to the bottom but lacks safe area handling.
- `LibraryView` only displays a list of exercises and a search bar.
- `WorkoutView` defaults to `StartSessionView` if no session is active, which includes exercise creation logic.
- `WorkoutStateManager` handles session starts but is currently tied to the `StartSessionView` inputs.

## Technical Constraints

- Dioxus 0.7 Signal-based reactivity.
- Tailwind CSS / DaisyUI for styling.
- PWA environment (must handle various mobile screen sizes and safe areas).
- Offline-first: all changes must persist to the SQLite database.

## Risks & Mitigations

- **UI Breaking Changes**: Moving the "Start Session" flow might confuse existing users (if any). Mitigation: Clear empty state in Workout tab directing to Library.
- **Safe Area Complexity**: Different devices have different safe areas. Mitigation: Use standard Tailwind safe area classes or CSS environment variables (`env(safe-area-inset-bottom)`).
