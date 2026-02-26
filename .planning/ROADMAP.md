# Roadmap: Simple Strength Assistant - File Picker Fix

## Overview

Fix the broken file picker that's preventing users from selecting where to store their workout database. We'll set up the development environment to capture errors, debug the File System Access API integration, fix the root cause, and verify the complete database initialization flow works reliably.

## Phases

- [x] **Phase 1: Development Environment** - Get app running locally with full debugging capabilities (completed 2026-02-26)
- [ ] **Phase 2: Debug and Fix File Picker** - Identify and fix the file picker issue
- [ ] **Phase 3: Verify and Polish** - Test complete flow and edge cases

## Phase Details

### Phase 1: Development Environment
**Goal**: Development environment runs with browser console access for debugging
**Depends on**: Nothing (first phase)
**Requirements**: DEV-01, DEV-02, DEV-03, DEV-04
**Success Criteria** (what must be TRUE):
  1. `dx serve` runs without errors and serves app in browser
  2. Browser loads the app and UI renders
  3. Console logs show initialization steps clearly
  4. WASM module loads successfully
**Plans**: 1 plan

Plans:
- [x] 01-01-PLAN.md — Initialize Dioxus logger and verify dev environment with console debugging

### Phase 2: Debug and Fix File Picker
**Goal**: File picker appears when triggered and user can select database file
**Depends on**: Phase 1
**Requirements**: DB-01, DB-02, DB-04, ERR-01, ERR-02, ERR-04
**Success Criteria** (what must be TRUE):
  1. File picker dialog appears when user needs to select database
  2. User can select .sqlite or .db file from filesystem
  3. File System Access API permission prompt appears correctly
  4. Selected file handle is accessible from Rust code
**Plans**: 2 plans

Plans:
- [x] 02-01-PLAN.md — Fix user gesture requirement by adding button to SelectingFile UI and preventing auto-prompt
- [ ] 02-02-PLAN.md — Add permission state verification for cached handles with queryPermission/requestPermission

### Phase 3: Verify and Polish
**Goal**: Complete database initialization flow works end-to-end with proper error handling
**Depends on**: Phase 2
**Requirements**: DB-03, DB-05, DB-06, ERR-03
**Success Criteria** (what must be TRUE):
  1. Selected file handle persists across browser refresh
  2. Database initializes successfully after file selection
  3. LocalStorage fallback works when API unavailable
  4. User-friendly error messages for common failure modes
**Plans**: TBD

Plans:
- [ ] 03-01: Test end-to-end flow and polish error handling

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Development Environment | 1/1 | Complete   | 2026-02-26 |
| 2. Debug and Fix File Picker | 1/2 | In Progress | - |
| 3. Verify and Polish | 0/1 | Not started | - |

---
*Roadmap created: 2026-02-25*
*Last updated: 2026-02-26 after completing Phase 2 Plan 1*
