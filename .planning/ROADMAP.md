# Roadmap: Simple Strength Assistant

## Milestones

- ✅ **v1.0 MVP** - Phases 1-3 (shipped 2026-02-26)
- 🚧 **v1.1 Exercise Library** - Phases 4-5 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-3) - SHIPPED 2026-02-26</summary>

### Phase 1: File Picker Foundation
**Goal**: Enable users to select and create database files via File System Access API
**Plans**: 3 plans

Plans:
- [x] 01-01: Implement file picker with user gesture detection
- [x] 01-02: Handle database creation and loading
- [x] 01-03: Add permission re-requesting for cached handles

### Phase 2: LocalStorage Fallback
**Goal**: Support browsers without File System Access API
**Plans**: 2 plans

Plans:
- [x] 02-01: Implement LocalStorage persistence layer
- [x] 02-02: Add browser compatibility detection

### Phase 3: PWA Deployment & Polish
**Goal**: Deploy installable PWA with polished error handling
**Plans**: 2 plans

Plans:
- [x] 03-01: Fix Vercel deployment and PWA installability
- [x] 03-02: Polish error UI with recovery instructions

</details>

### 🚧 v1.1 Exercise Library (In Progress)

**Milestone Goal:** Provide users with a centralized view to browse and search their exercise collection.

- [ ] **Phase 4: Tab Navigation Foundation** - Enable navigation between Workout and Library views
- [ ] **Phase 5: Exercise List & Search** - Browse and search exercises with type indicators

## Phase Details

### Phase 4: Tab Navigation Foundation
**Goal**: Users can navigate between Workout and Library tabs without losing active workout session state
**Depends on**: Phase 3
**Requirements**: LIB-01, LIB-02
**Success Criteria** (what must be TRUE):
  1. User can see "Workout" and "Library" tabs in the main interface
  2. User can click Library tab and see placeholder content
  3. User can switch back to Workout tab and see active session preserved (exercises, sets, timer state)
  4. Tab selection persists when user refreshes browser (active tab restored)
**Plans**: 2 plans

Plans:
- [ ] 04-01-PLAN.md — BDD test scaffolding (feature files and step definitions)
- [ ] 04-02-PLAN.md — Tab navigation implementation (TabBar, view components, conditional rendering)

### Phase 5: Exercise List & Search
**Goal**: Users can browse all exercises they've created and search by name with instant filtering
**Depends on**: Phase 4
**Requirements**: LIB-03, LIB-04, LIB-05, LIB-06
**Success Criteria** (what must be TRUE):
  1. User sees all exercises they've created in a scrollable list when Library tab is active
  2. User sees exercise type badge (weighted vs bodyweight) for each exercise in the list
  3. User can type in search box and see exercise list filter instantly as they type
  4. User sees clear empty state message "No exercises yet. Add exercises during your first workout." when no exercises exist
  5. User sees "No matching exercises" when search returns zero results
**Plans**: TBD

Plans:
- [x] 05-01-PLAN.md — BDD Feature Files and Test Harness Setup
- [x] 05-02-PLAN.md — Exercise List Component and Type Badges

## Progress

**Execution Order:**
Phases execute in numeric order: 4 → 5

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. File Picker Foundation | v1.0 | 3/3 | Complete | 2026-02-26 |
| 2. LocalStorage Fallback | v1.0 | 2/2 | Complete | 2026-02-26 |
| 3. PWA Deployment & Polish | v1.0 | 2/2 | Complete | 2026-02-26 |
| 4. Tab Navigation Foundation | v1.1 | 0/2 | Ready | - |
| 5. Exercise List & Search | v1.1 | 0/TBD | Not started | - |
