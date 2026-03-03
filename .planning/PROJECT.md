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
- ✓ User can view all exercises in a dedicated Exercise Library tab — v1.1
- ✓ User can see exercise metadata (last performed, total sessions, performance indicators) — v1.1
- ✓ User can search and filter exercises by name — v1.1
- ✓ User can edit exercise details (name, settings) — v1.1
- ✓ User can archive exercises to prevent orphaned workout data — v1.1

### Active

- [ ] User can specify a Minimum Weight for each exercise
- [ ] Application uses Minimum Weight as fallback for session suggestions
- [ ] Application no longer uses the 'Starting Weight' concept

### Out of Scope

- Cloud sync — Offline-first philosophy, user owns data locally
- Multi-device sync — Requires cloud infrastructure
- Social features — Not relevant to personal workout tracking
- Workout programs library — Defer until prescription system exists
- Exercise deletion — Archive instead to prevent orphaned workout references
- Exercise categorization by muscle group — Keep simple for v1.1, defer to future
- Database export/import — Deferred to future milestone
- Tactile input components (tape measure, RPE slider) — Previous v1.1 scope, deferred
- Workout prescription based on history beyond last recorded weight — Deferred to future milestone

## Current State (v1.1)

**Shipped:** 2026-03-02
**Status:** Exercise Library tab and management capabilities are fully functional and verified.

**What works:**
- Dedicated Exercise Library tab with list and search.
- Editing exercise details and archiving exercises.
- Full database lifecycle: selection, creation, persistence, and loading.
- Native File System Access API integration with automatic permission handling.
- Cross-browser compatibility: Chrome/Edge (File System Access), Safari/Firefox (LocalStorage fallback).
- PWA installability on mobile (Vercel deployment fixed).
- Reactive UI state using Dioxus 0.7 Signals.
- User-friendly error handling with recovery instructions.

## Current Milestone: v1.2 Minimum Weight

**Goal:** Allow users to specify a minimum weight for exercises, ensuring weight suggestions are actionable, and replace the unused 'starting weight' concept.

**Target features:**
- New Input Field for Minimum Weight on exercise forms
- Remove references and UI for 'Starting Weight'
- Determine suggestions using previous weight or fallback to Minimum Weight

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| File System Access API | User controls data location, enables backup/sharing | ✓ Good - fixed and verified |
| SQLite via sql.js WASM | Full SQL queries, mature ecosystem, portable format | ✓ Good - works well |
| Dioxus framework | Rust safety + React-like patterns + WASM target | ✓ Good - signal reactivity working |
| Inline initialization | Eliminated fragile error message string matching | ✓ Good - simplified flow |
| User gesture button | Browser security requirement for file picker | ✓ Good - required for API |
| Minimum weight over starting weight | Makes suggestions actionable directly (e.g. barbell weight) | — Pending |

---
*Last updated: 2026-03-03 after v1.2 milestone start*