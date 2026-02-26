# Simple Strength Assistant

## What This Is

An offline-first PWA for tracking workouts with intelligent prescription. Users log exercises with RPE-based auto-regulation, and the app suggests weight/reps based on performance history. All data stored locally in a SQLite database at a user-selected filesystem location.

## Core Value

Users must be able to reliably persist their workout data to a file they control on their device. Without working database setup, no other features matter.

## Requirements

### Validated

- ✓ User can start workout session with exercise metadata — existing
- ✓ User can log completed sets with reps, weight/bodyweight, and RPE — existing
- ✓ User can complete workout sessions — existing
- ✓ Database schema supports sessions, exercises, and completed sets — existing
- ✓ App compiles to WASM and runs as PWA — existing
- ✓ File picker shows via user gesture and user can select .sqlite/.db file — v1.0
- ✓ File handle persists across browser sessions via IndexedDB — v1.0
- ✓ Permission re-requesting for cached handles — v1.0
- ✓ LocalStorage fallback works for unsupported browsers — v1.0
- ✓ PWA installable and functional on mobile Chrome — v1.0

### Active

- [ ] User can export database to JSON format
- [ ] User can import data from previous exports
- [ ] User can vacuum/optimize database file
- [ ] Workout prescription based on history (suggest weight/reps)
- [ ] Historical data view showing past sessions

### Out of Scope

- Cloud sync — Offline-first philosophy, user owns data locally
- Multi-device sync — Requires cloud infrastructure
- Social features — Not relevant to personal workout tracking
- Workout programs library — Defer until prescription system exists

## Current State (v1.0)

**Shipped:** 2026-02-26
**Status:** Core infrastructure and database persistence layer are fully functional and verified.

**What works:**
- Full database lifecycle: selection, creation, persistence, and loading.
- Native File System Access API integration with automatic permission handling.
- Cross-browser compatibility: Chrome/Edge (File System Access), Safari/Firefox (LocalStorage fallback).
- PWA installability on mobile (Vercel deployment fixed).
- Reactive UI state using Dioxus 0.7 Signals.
- User-friendly error handling with recovery instructions.

## Next Milestone Goals (v1.1)

- Implement data management features (Export/Import/Vacuum).
- Build the historical data view and progress visualization.
- Begin work on the intelligent workout prescription system.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| File System Access API | User controls data location, enables backup/sharing | ✓ Good - fixed and verified |
| SQLite via sql.js WASM | Full SQL queries, mature ecosystem, portable format | ✓ Good - works well |
| Dioxus framework | Rust safety + React-like patterns + WASM target | ✓ Good - signal reactivity working |
| Inline initialization | Eliminated fragile error message string matching | ✓ Good - simplified flow |
| User gesture button | Browser security requirement for file picker | ✓ Good - required for API |

---
*Last updated: 2026-02-26 after v1.0 milestone*
