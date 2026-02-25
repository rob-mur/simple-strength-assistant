# Simple Strength Assistant

## What This Is

An offline-first PWA for tracking workouts with intelligent prescription. Users log exercises with RPE-based auto-regulation, and the app suggests weight/reps based on performance history. All data stored locally in a SQLite database at a user-selected filesystem location.

## Core Value

Users must be able to reliably persist their workout data to a file they control on their device. Without working database setup, no other features matter.

## Requirements

### Validated

- ✓ User can start workout session with exercise metadata (name, set type, default reps) — existing
- ✓ User can log completed sets with reps, weight/bodyweight, and RPE — existing
- ✓ User can complete workout sessions — existing
- ✓ Database schema supports sessions, exercises, and completed sets — existing
- ✓ App compiles to WASM and runs as PWA — existing

### Active

- [ ] File picker shows when user needs to select database location
- [ ] User can successfully select a .sqlite or .db file from filesystem
- [ ] Dev environment is runnable to capture browser console logs
- [ ] File System Access API errors are properly handled and logged
- [ ] Database initialization completes without blocking on file picker

### Out of Scope

- Workout prescription/suggestion system — future milestone
- Historical data view and progress tracking — future milestone
- Program structure and progression rules — future milestone
- Data export/backup functionality — future milestone
- Multi-device sync — future milestone

## Context

**Current blocker:** File picker is not showing up when it should. Browser console errors are preventing the File System Access API from working correctly. This blocks all database functionality since users can't select where to store their data.

**Technical environment:**
- Dioxus 0.7 (Rust web framework)
- sql.js (SQLite compiled to WASM)
- File System Access API (primary) with LocalStorage fallback
- Deployed as static PWA on Vercel

**What works:**
- App compiles and loads in browser
- UI components render (StartSessionView, ActiveSession)
- In-memory database operations
- Validation and state management

**What's broken:**
- File picker doesn't appear when user clicks "Select Database File" (or equivalent trigger)
- Console shows errors related to File System Access API or file handle permissions
- Users can't complete database initialization flow

## Constraints

- **Platform:** Must remain offline-first PWA, no backend server
- **Storage:** Must use File System Access API as primary storage (LocalStorage is fallback only)
- **Browser support:** Modern browsers with WASM support (Chrome, Edge, Safari)
- **Performance:** Database operations must complete in <500ms for good UX

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| File System Access API over IndexedDB | User controls data location, enables backup/sharing | — Pending - currently broken |
| SQLite via sql.js WASM | Full SQL queries, mature ecosystem, portable format | ✓ Good - works well |
| Dioxus framework | Rust safety + React-like patterns + WASM target | ✓ Good - type safety catching bugs |

---
*Last updated: 2026-02-25 after initialization*
