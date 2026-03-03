# Project Research Summary

**Project:** Simple Strength Assistant - Exercise Library (v1.1)
**Domain:** Exercise management UI for workout tracking PWA
**Researched:** 2026-03-01
**Confidence:** HIGH

## Executive Summary

The Exercise Library milestone is a low-risk feature addition that requires **zero new dependencies** and integrates cleanly into the existing Dioxus 0.7 + SQLite architecture. Research confirms that all required capabilities already exist in the validated v1.0 stack: reactive list rendering via Dioxus signals, client-side search filtering, SQLite CRUD operations, and DaisyUI components. The feature adds a parallel view to the active workout session using tab-based conditional rendering, enabling users to browse, search, and manage their personal exercise library without disrupting active workout sessions.

The recommended approach is incremental development across 3 phases: (1) tab navigation foundation, (2) exercise list with search, and (3) exercise metadata display. This ordering avoids the critical pitfalls identified in research: foreign key enforcement failures, reactive signal cascades, and poor empty state UX. Two advanced features—edit exercise and archive exercise—should be deferred to future milestones due to transaction complexity and data migration requirements.

Key risks are minimal for the core functionality. The main architectural decision is whether to use exercise_id vs exercise_name as foreign keys in the sessions table. The current schema uses exercise_name (text), which simplifies queries but creates data integrity challenges for renames. Research strongly recommends migrating to exercise_id foreign keys if edit functionality is planned, but this can be deferred if v1.1 ships as read-only library.

## Key Findings

### Recommended Stack

The v1.0 stack remains sufficient—no new dependencies required. Research validates that Dioxus 0.7.2 reactive signals handle list rendering and search filtering efficiently for expected scale (5-100 exercises). SQLite via sql.js 1.13.0 provides all necessary query capabilities: LEFT JOIN for exercise stats, LIKE for search, ALTER TABLE for archive column addition. DaisyUI 4.12.14 already includes all UI components needed: cards for exercise list items, inputs for search, badges for metadata display.

**Core technologies:**
- **Dioxus 0.7.2**: Reactive lists, forms, search filtering — native `for` loops in `rsx!` macro, `use_signal` for state management
- **SQLite (sql.js) 1.13.0**: Full CRUD, aggregate queries, ALTER TABLE support — latest version with SQLite 3.49 core
- **DaisyUI 4.12.14**: Cards, inputs, buttons, badges, tables — already used throughout existing UI
- **Tailwind CSS 3.4.17**: Styling, responsive layout — zero-setup with Dioxus 0.7

**Critical version note:** All dependencies locked at current versions. No bumps needed or recommended.

### Expected Features

Research shows that exercise library features fall into clear tiers based on competitive analysis and user expectations from fitness apps.

**Must have (table stakes):**
- **Exercise list view** — users expect to browse exercises they've created
- **Search by name** — instant text filtering prevents endless scrolling (complexity is LOW, no excuse to skip)
- **Last performed date** — progressive overload requires knowing "what did I do last time"
- **Total sessions count** — users track consistency ("I've benched 47 times this year")
- **Archive exercise** — safer than deletion, preserves workout history integrity

**Should have (competitive advantage):**
- **Previous session stats** — show last weight/reps from most recent workout to plan progressive overload
- **Personal records tracking** — auto-detect and highlight PRs (heaviest weight, most reps)
- **Quick-add from library to workout** — tap exercise in library to immediately add to active session

**Defer (v2+):**
- **Exercise usage heatmap** — complex visualization, defer until users request training balance insights
- **Performance indicators (volume trend)** — derivative metric requiring date-based aggregation
- **Edit exercise name** — requires transaction to migrate session references (complex for MVP)

**Anti-features to avoid:**
- **Pre-populated exercise database** — bloats initial load, wrong for user's naming preferences
- **Exercise categorization by muscle group** — adds complexity without validation, users won't maintain tags
- **Video/image tutorials** — massive storage overhead, scope creep (user can Google form)

### Architecture Approach

The Exercise Library integrates as a parallel view to the active workout session using tab-based conditional rendering. No router needed since PWA remains simple (2 views: workout vs library). State management extends existing WorkoutState with `active_tab: Signal<Tab>` field. Database queries are read-only for exercise list/metadata, reusing existing ExerciseMetadata model.

**Major components:**
1. **MainInterface** — top-level tab coordinator, replaces WorkoutInterface in render tree
2. **TabBar** — tab selection UI, writes to WorkoutState.active_tab on click
3. **ExerciseLibraryTab** — exercise list view with search, uses `use_resource` for async database queries
4. **ExerciseCard** — display single exercise with metadata (last performed, total sessions)
5. **WorkoutTab** — existing WorkoutInterface renamed, nested under MainInterface

**Key architectural patterns:**
- **Tab-based conditional rendering** — use Signal<Tab> with match expression, simpler than router for 2-3 views
- **use_resource for async queries** — Dioxus hook handles loading/error states, integrates with signal reactivity
- **LEFT JOIN for metadata** — single SQL query per exercise to get last performed date and session count
- **Shared context via use_context** — WorkoutState passed to nested components, avoids prop drilling

**Database schema status:** Existing schema already supports exercise library queries. The `exercises` table contains all metadata, `sessions.exercise_name` enables stats aggregation via JOIN. No schema changes needed for Phase 1-2 (list and metadata). Phase 3+ (archive) requires adding `archived_at INTEGER` column.

### Critical Pitfalls

Research identified 10 critical pitfalls specific to exercise library features in workout tracking apps. Top 5 by severity:

1. **Archive without foreign key enforcement** — SQLite has foreign keys DISABLED by default. Must execute `PRAGMA foreign_keys = ON;` immediately after opening database connection (every session), not just in init code. Without this, archiving exercises can orphan workout data silently. Prevention: Add FK enforcement tests that verify constraint violations throw errors.

2. **Reactive signal cascades on every keystroke** — Connecting search input directly to reactive signals without debouncing triggers full re-renders and database queries on every keystroke, causing 13x more signal effects than necessary. Prevention: Debounce search input 300-500ms before writing to signals, use `.peek()` for reading signals in event handlers without subscribing.

3. **Exercise rename breaking workout history references** — Current schema stores `exercise_name TEXT` in sessions table. Renaming "Squat" to "Back Squat" leaves sessions referencing old name, making historical data invisible. Prevention: Either use `exercise_id` foreign key instead of name, or wrap renames in transaction that updates both exercises.name and all sessions.exercise_name atomically.

4. **Case-insensitive search without proper indexing** — Implementing search using `LOWER(name) LIKE ?` without indexed support causes full table scans, 2-5x performance degradation with 100+ exercises. Prevention: Use `COLLATE NOCASE` on name column definition and create index with NOCASE collation, or accept client-side filtering for small datasets (<100 exercises).

5. **No empty state for first-time users** — Blank exercise list confuses new users who don't understand they need to add exercises during workouts first. Prevention: Detect empty list and show purpose-built empty state with clear "Add exercises during your first workout" guidance.

**Additional noteworthy pitfalls:**
- Partial updates from failed transactions (sql.js persistence model differs from server SQLite)
- Search matching archived exercises by default (must filter WHERE archived_at IS NULL)
- Memory bloat from sql.js in-memory database (monitor size, test with 10,000+ sets)
- Archive UI that feels like permanent delete (unclear UX leads to user anxiety)

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Tab Navigation Foundation
**Rationale:** Establish UI structure and state management before implementing features. Lowest risk phase—minimal code changes to existing working system.

**Delivers:** User can switch between "Workout" and "Library" tabs without losing active session state. Library tab shows placeholder "Coming soon" message.

**Addresses Features:** None yet—pure infrastructure.

**Avoids Pitfalls:**
- No empty state pitfall (addressed from start with placeholder)
- State management foundation for preventing signal cascades later

**Estimated Effort:** 2-3 hours

**Research Flag:** Standard pattern—no additional research needed. Dioxus signals and conditional rendering are well-documented.

---

### Phase 2: Exercise List with Search
**Rationale:** Core value delivery—users can browse their exercises. Search is table stakes (not optional) per competitive analysis. Client-side filtering sufficient for expected scale.

**Delivers:** User sees all exercises in list view, can search by name with instant results, sees exercise type (weighted vs bodyweight).

**Addresses Features:**
- Exercise list view (table stakes)
- Search by name (table stakes)

**Avoids Pitfalls:**
- Case-insensitive search without indexing (client-side filtering for MVP, defer database indexing)
- Reactive signal cascades (implement debouncing from start)
- Empty state UX (show clear "No exercises yet" with guidance)

**Uses Stack:**
- Database.list_exercises() query (new method, existing pattern)
- use_resource hook for async loading
- DaisyUI cards for exercise list items

**Estimated Effort:** 3-4 hours

**Research Flag:** Standard pattern—list rendering and search are well-documented. No additional research needed.

---

### Phase 3: Exercise Metadata Display
**Rationale:** Differentiator feature—shows last performed date and session count, essential for progressive overload tracking. Enables users to plan workouts based on history.

**Delivers:** Each exercise card shows when last performed and total session count. Enables users to quickly see "I benched 3 days ago, did 12 sessions this year."

**Addresses Features:**
- Last performed date (table stakes)
- Total sessions count (table stakes)
- Previous session stats (differentiator—partial implementation)

**Avoids Pitfalls:**
- Foreign key enforcement (verify PRAGMA in tests before shipping metadata)
- N+1 query problem (acceptable for MVP with <50 exercises, note optimization path)

**Uses Stack:**
- Database.get_exercise_stats() with LEFT JOIN query (new method)
- ExerciseStats struct for aggregated data
- Date formatting utilities

**Estimated Effort:** 2-3 hours

**Research Flag:** Standard SQL aggregation pattern. No additional research needed.

---

### Phase 4: Edit Exercise (DEFERRED)
**Rationale:** High complexity relative to value for MVP. Editing exercise name requires transaction to update both exercises table and all sessions.exercise_name references. Read-only library sufficient for v1.1 validation.

**Defer to:** v1.2 or later, after core library usage validated

**Complexity Factors:**
- Schema decision: migrate to exercise_id foreign keys vs continue with exercise_name
- Transaction wrapping to prevent partial updates
- Validation to prevent duplicate names
- Modal component infrastructure

**Research Flag:** Requires phase-specific research for transaction patterns and data migration strategies.

---

### Phase 5: Archive Exercise (DEFERRED)
**Rationale:** Requires database schema migration (add archived_at column) and filtered queries across codebase. UX complexity around archive/unarchive flows. Better suited for dedicated milestone after core library proven.

**Defer to:** v1.2 or later

**Complexity Factors:**
- Database migration strategy for adding archived_at column
- All list queries need WHERE archived_at IS NULL filter
- Unarchive functionality for user mistakes
- Clear archive UX to prevent confusion with deletion

**Research Flag:** Requires phase-specific research for soft delete patterns and migration approaches.

---

### Phase Ordering Rationale

**Why Foundation → List → Metadata:**
- Foundation establishes state management patterns that List phase depends on
- List provides visible value quickly (2-3 hours to working UI)
- Metadata builds on List without requiring architectural changes
- Each phase is independently testable and shippable

**Why defer Edit and Archive:**
- Both require schema decisions (exercise_id vs exercise_name foreign keys)
- Edit requires transaction patterns not yet proven in codebase
- Archive requires schema migration (ALTER TABLE) and filtered queries
- MVP can ship without these—users edit via workout creation flow
- Deferring reduces risk and enables faster validation of core library usage

**Dependency chain:**
```
Phase 1 (Foundation)
    ↓ (tab structure needed)
Phase 2 (List + Search)
    ↓ (exercise data needed)
Phase 3 (Metadata)
    ↓ (read-only validation complete)
Phase 4 (Edit) — DEFERRED
    ↓ (edit capability needed)
Phase 5 (Archive) — DEFERRED
```

### Research Flags

**Phases with standard patterns (skip research-phase):**
- **Phase 1:** Tab navigation—well-documented Dioxus signal pattern
- **Phase 2:** List rendering and search—standard Dioxus component patterns
- **Phase 3:** SQL aggregation queries—standard SQLite LEFT JOIN pattern

**Phases needing deeper research (if implemented):**
- **Phase 4 (Edit):** Transaction patterns in sql.js WASM context differ from server SQLite
- **Phase 5 (Archive):** Schema migration strategies for browser-based SQLite

**All phases 1-3 can proceed without `/gsd:research-phase` calls.** Architecture is straightforward extension of existing patterns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All capabilities verified in existing v1.0 codebase. Zero new dependencies needed. |
| Features | MEDIUM | Based on web search of 2026 fitness app patterns. No official domain standards exist, but competitive analysis clear. |
| Architecture | HIGH | Integration points identified in existing codebase. Patterns proven (signals, use_resource, DaisyUI). |
| Pitfalls | HIGH | Sourced from SQLite official docs, Dioxus GitHub issues, and real-world fitness app development blogs. |

**Overall confidence:** HIGH for Phases 1-3, MEDIUM for deferred Phases 4-5 (edit/archive need deeper research if implemented)

### Gaps to Address

**Schema decision for future edit functionality:**
- Current schema uses `exercise_name TEXT` in sessions table (denormalized)
- Research recommends `exercise_id INTEGER` foreign key for data integrity
- Gap: Need to decide migration path before implementing edit feature
- Handle during: Phase 4 planning (if/when edit is scheduled)

**Transaction patterns in WASM SQLite:**
- sql.js operates entirely in-memory, changes persist only when explicitly exported to file
- Gap: Transaction + export timing unclear—does COMMIT guarantee persistence?
- Handle during: Phase 3 or 4 testing with simulated browser crashes

**Scalability threshold for client-side search:**
- Research assumes <100 exercises for client-side filtering
- Gap: When to migrate to database-side search with indexed COLLATE NOCASE?
- Handle during: Phase 2 implementation—add database size monitoring, flag at 50+ exercises

**Empty state messaging:**
- New users need to understand exercise library populates from workout sessions
- Gap: Exact wording and CTA unclear (where does "Add Exercise" link to?)
- Handle during: Phase 2 UX design—likely link to StartSessionView

**Foreign key enforcement verification:**
- SQLite disables FK constraints by default, requires PRAGMA per session
- Gap: Existing codebase verification needed—is PRAGMA already set?
- Handle during: Phase 1 implementation—audit db.rs for FK enforcement

## Sources

### Primary (HIGH confidence)

**Stack capabilities:**
- Existing v1.0 codebase (src/) — verified Dioxus 0.7.2, sql.js 1.13.0, DaisyUI 4.12.14 in use
- [Dioxus 0.7 Components Documentation](https://dioxuslabs.com/learn/0.7/essentials/ui/components/) — list rendering patterns
- [Dioxus 0.7.0 Release](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0) — signal reactivity features
- [sql.js GitHub Repository](https://github.com/sql-js/sql.js) — version 1.13.0 capabilities

**Database patterns:**
- [SQLite Foreign Key Support](https://sqlite.org/foreignkeys.html) — FK enforcement documentation
- [SQLite Atomic Commit](https://sqlite.org/atomiccommit.html) — transaction guarantees
- [SQLite JOIN Tutorial](https://www.sqlitetutorial.net/sqlite-join/) — aggregate query patterns
- [SQLite Aggregate Functions](https://sqlite.org/lang_aggfunc.html) — official built-in functions

**Architecture patterns:**
- [Dioxus Signals State Management](https://dioxuslabs.com/learn/0.7/essentials/basics/signals/) — reactive state
- [Dioxus Navigation Documentation](https://dioxuslabs.com/learn/0.7/essentials/router/navigation/) — routing vs conditional rendering

**Pitfalls verification:**
- [Dioxus Signal Issues #4039](https://github.com/DioxusLabs/dioxus/issues/4039) — ErrorBoundary reactivity bugs
- [sql.js Out of Memory Issues #574](https://github.com/sql-js/sql.js/issues/574) — memory leak investigation
- [Notion's WASM SQLite Performance](https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite) — real-world WASM optimization

### Secondary (MEDIUM confidence)

**Feature expectations:**
- [JEFIT Best Workout Apps 2026](https://www.jefit.com/wp/guide/best-workout-apps-for-2026-top-options-tested-and-reviewed-by-pro/) — competitive analysis
- [Fitness App UX Design Principles](https://stormotion.io/blog/fitness-app-ux/) — user expectations
- [Best Workout Tracker App 2026](https://www.hevyapp.com/best-workout-tracker-app/) — feature benchmarking
- [7 Things People Hate in Fitness Apps](https://www.ready4s.com/blog/7-things-people-hate-in-fitness-apps) — anti-patterns

**UX patterns:**
- [Empty State UX Best Practices](https://www.eleken.co/blog-posts/empty-state-ux) — empty state design
- [Empty States in User Onboarding](https://www.useronboard.com/onboarding-ux-patterns/empty-states/) — first-run experience
- [Archive/Unarchive Implementation](https://socialrails.com/blog/how-to-archive-unarchive-instagram-posts) — user expectations

**Performance considerations:**
- [SQLite Persistence on Web - November 2025](https://www.powersync.com/blog/sqlite-persistence-on-the-web) — browser persistence options
- [React Debouncing Guide](https://www.developerway.com/posts/debouncing-in-react) — debounce patterns (React, but applicable)

### Tertiary (LOW confidence)

**Domain context:**
- [Fitness App Development Challenges](https://codiant.com/blog/challenges-in-building-a-fitness-app/) — general guidance
- [2026 Fitness Trends](https://www.feed.fm/2026-digital-fitness-ecosystem-report) — user expectations context
- [Avoiding Soft Delete Anti-Pattern Discussion](https://news.ycombinator.com/item?id=40326815) — community debate on archive patterns

---
*Research completed: 2026-03-01*
*Ready for roadmap: yes*
