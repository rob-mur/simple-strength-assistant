# Exercise Library Architecture Integration

**Project:** Simple Strength Assistant - Exercise Library (v1.1)
**Researched:** 2026-03-01
**Confidence:** HIGH

## Executive Summary

The Exercise Library integrates into the existing Dioxus 0.7 + SQLite architecture as a **parallel view** to the active workout session. The integration uses existing infrastructure (WorkoutState, Database, Signals) with minimal architectural changes. Key pattern: **tab-based conditional rendering** without router, **read-only database queries** for exercise list/metadata, **reuse existing ExerciseMetadata model**.

**Integration Strategy:** Extend existing single-page architecture with conditional tab rendering. No router needed since the PWA is simple (2 views: workout vs library). State management via existing Dioxus 0.7 Signals pattern. Database schema already supports exercise library queries via `exercises` table.

## Recommended Architecture

### High-Level Structure

```
App (root)
├── WorkoutState (existing Signal-based context)
├── Database (existing SQLite wrapper)
├── Storage (existing file system manager)
└── MainInterface (NEW - replaces WorkoutInterface)
    ├── TabBar (NEW)
    ├── WorkoutTab (existing WorkoutInterface renamed)
    │   ├── StartSessionView
    │   └── ActiveSession
    └── ExerciseLibraryTab (NEW)
        ├── ExerciseList
        ├── ExerciseCard
        └── ExerciseEditModal (optional Phase 2+)
```

### Integration Points with Existing Architecture

| Existing Component | Integration Point | Change Type |
|-------------------|-------------------|-------------|
| `WorkoutState` | Add `active_tab: Signal<Tab>` field | **Extend** - minimal change |
| `Database` | Add `list_exercises()`, `get_exercise_stats()` methods | **Extend** - new read queries |
| `ExerciseMetadata` | Used as-is for exercise data | **Reuse** - no changes |
| `WorkoutInterface` | Renamed to `WorkoutTab`, nested under `MainInterface` | **Refactor** - structure change |
| `App` component | Replace `WorkoutInterface` with `MainInterface` | **Modify** - single line change |

## Component Boundaries

### NEW Components

#### MainInterface
**Responsibility:** Top-level tab coordinator
**State:** Reads `WorkoutState.active_tab` signal
**Communicates With:** TabBar, WorkoutTab, ExerciseLibraryTab

```rust
#[component]
fn MainInterface(state: WorkoutState) -> Element {
    let active_tab = state.active_tab(); // Signal<Tab>

    rsx! {
        div {
            TabBar { state }
            match active_tab {
                Tab::Workout => rsx! { WorkoutTab { state } },
                Tab::Library => rsx! { ExerciseLibraryTab { state } }
            }
        }
    }
}
```

#### TabBar
**Responsibility:** Tab selection UI
**State:** Writes to `WorkoutState.active_tab` on click
**Communicates With:** WorkoutState (setter only)

```rust
#[component]
fn TabBar(state: WorkoutState) -> Element {
    let active_tab = state.active_tab();

    rsx! {
        div { class: "tabs tabs-bordered",
            button {
                class: if active_tab == Tab::Workout { "tab tab-active" } else { "tab" },
                onclick: move |_| state.set_active_tab(Tab::Workout),
                "Workout"
            }
            button {
                class: if active_tab == Tab::Library { "tab tab-active" } else { "tab" },
                onclick: move |_| state.set_active_tab(Tab::Library),
                "Library"
            }
        }
    }
}
```

#### ExerciseLibraryTab
**Responsibility:** Exercise list view with search/filter
**State:** Local `search_query: Signal<String>`, reads `Database` via `WorkoutState`
**Communicates With:** Database (read-only), ExerciseCard (child components)

```rust
#[component]
fn ExerciseLibraryTab(state: WorkoutState) -> Element {
    let mut search_query = use_signal(|| String::new());
    let exercises = use_resource(move || {
        let db = state.database().unwrap();
        async move { db.list_exercises().await }
    });

    rsx! {
        div {
            SearchInput { value: search_query }
            match exercises.read_unchecked() {
                Some(Ok(list)) => rsx! {
                    for exercise in list.iter().filter(|e| matches_search(e, &search_query())) {
                        ExerciseCard { exercise: exercise.clone(), state }
                    }
                },
                Some(Err(e)) => rsx! { ErrorDisplay { error: e } },
                None => rsx! { LoadingSpinner {} }
            }
        }
    }
}
```

#### ExerciseCard
**Responsibility:** Display single exercise with metadata
**State:** Reads exercise stats via `use_resource` (last performed, total sessions)
**Communicates With:** Database (read-only), ExerciseEditModal (future)

```rust
#[component]
fn ExerciseCard(exercise: ExerciseMetadata, state: WorkoutState) -> Element {
    let stats = use_resource(move || {
        let db = state.database().unwrap();
        let name = exercise.name.clone();
        async move { db.get_exercise_stats(&name).await }
    });

    rsx! {
        div { class: "card bg-base-100 shadow",
            div { class: "card-body",
                h3 { class: "card-title", "{exercise.name}" }
                if let Some(Ok(s)) = stats.read_unchecked() {
                    p { "Last performed: {s.last_performed_date}" }
                    p { "Total sessions: {s.total_sessions}" }
                }
            }
        }
    }
}
```

### MODIFIED Components

#### WorkoutState (extend)
**NEW Fields:**
```rust
pub struct WorkoutState {
    // ... existing fields ...
    active_tab: Signal<Tab>, // NEW
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Workout,
    Library,
}

impl WorkoutState {
    pub fn active_tab(&self) -> Tab {
        (self.active_tab)()
    }

    pub fn set_active_tab(&self, tab: Tab) {
        let mut sig = self.active_tab;
        sig.set(tab);
    }
}
```

#### Database (extend)
**NEW Methods:**
```rust
impl Database {
    // Phase 1: List all exercises
    pub async fn list_exercises(&self) -> Result<Vec<ExerciseMetadata>, DatabaseError> {
        let sql = "SELECT name, is_weighted, min_weight, increment FROM exercises ORDER BY name";
        // ... execute and map to ExerciseMetadata
    }

    // Phase 2: Exercise stats for metadata display
    pub async fn get_exercise_stats(&self, exercise_name: &str) -> Result<ExerciseStats, DatabaseError> {
        let sql = r#"
            SELECT
                e.name,
                MAX(s.completed_at) as last_performed,
                COUNT(DISTINCT s.id) as total_sessions
            FROM exercises e
            LEFT JOIN sessions s ON e.name = s.exercise_name AND s.completed_at IS NOT NULL
            WHERE e.name = ?
            GROUP BY e.name
        "#;
        // ... execute and map to ExerciseStats
    }

    // Phase 3: Archive exercise (soft delete)
    pub async fn archive_exercise(&self, exercise_name: &str) -> Result<(), DatabaseError> {
        // Add archived_at column to exercises table (migration)
        let sql = "UPDATE exercises SET archived_at = ? WHERE name = ?";
        // ... execute with current timestamp
    }

    // Phase 4: Update exercise
    pub async fn update_exercise(&self, old_name: &str, exercise: &ExerciseMetadata) -> Result<(), DatabaseError> {
        let sql = "UPDATE exercises SET name = ?, is_weighted = ?, min_weight = ?, increment = ? WHERE name = ?";
        // ... execute with parameters
    }
}

pub struct ExerciseStats {
    pub name: String,
    pub last_performed: Option<i64>, // timestamp or None if never performed
    pub total_sessions: i64,
}
```

## Data Flow

### Exercise List Loading Flow

```
User clicks "Library" tab
    ↓
TabBar updates WorkoutState.active_tab = Tab::Library
    ↓
MainInterface re-renders, shows ExerciseLibraryTab
    ↓
ExerciseLibraryTab use_resource triggers
    ↓
Database.list_exercises() executes SQL query
    ↓
ExerciseMetadata list returned
    ↓
ExerciseLibraryTab renders ExerciseCard for each
    ↓
Each ExerciseCard use_resource triggers
    ↓
Database.get_exercise_stats() executes per exercise
    ↓
Stats displayed in card
```

**Performance Note:** Initial implementation uses `use_resource` per card. If >50 exercises, optimize by batch-loading stats in ExerciseLibraryTab instead.

### Search/Filter Flow

```
User types in search input
    ↓
search_query Signal updates
    ↓
ExerciseLibraryTab re-renders (reactive)
    ↓
filter() applied to exercise list (client-side)
    ↓
Matching ExerciseCard components rendered
```

**Architecture Decision:** Client-side filtering sufficient for v1.1 (expected <100 exercises). Database query filtering deferred to future optimization.

### Tab Switching with Active Session

```
Active workout session exists
    ↓
User clicks "Library" tab
    ↓
WorkoutState.current_session PERSISTS (not cleared)
    ↓
User can browse library while session active
    ↓
User clicks "Workout" tab
    ↓
ActiveSession re-renders with existing session
```

**Rationale:** Session state independent of UI tab. User can reference library mid-workout without losing session.

## State Management Approach

### Signal-Based Reactivity (Existing Pattern)

Dioxus 0.7 uses Signals for reactive state. **Pattern from existing code:**

```rust
// WorkoutState fields are all Signal<T>
pub struct WorkoutState {
    current_session: Signal<Option<WorkoutSession>>,
    database: Signal<Option<Database>>,
    // ... etc
}

// Reading triggers reactive subscription
let session = state.current_session();

// Writing triggers re-render of subscribed components
state.set_current_session(Some(new_session));
```

### Exercise Library State Pattern

```rust
// Local component state (NOT shared)
let mut search_query = use_signal(|| String::new());

// Shared app state (via WorkoutState context)
let active_tab = state.active_tab(); // Signal read

// Async data loading (use_resource pattern)
let exercises = use_resource(move || async move {
    state.database().unwrap().list_exercises().await
});
```

**Key Principle:** Database queries are asynchronous. Use `use_resource` hook for reactive async data loading. Automatically re-runs when dependencies change.

## Patterns to Follow

### Pattern 1: Tab-Based Conditional Rendering
**What:** Use Signal<Tab> with match expression for view switching
**When:** Simple PWA with 2-3 views, no URL routing needed
**Why:** Simpler than router for basic tab navigation, maintains app state across tabs

**Example:**
```rust
match state.active_tab() {
    Tab::Workout => rsx! { WorkoutTab { state } },
    Tab::Library => rsx! { ExerciseLibraryTab { state } }
}
```

**Source:** [Dioxus Signals Documentation](https://dioxuslabs.com/learn/0.7/essentials/basics/signals/)

### Pattern 2: use_resource for Async Database Queries
**What:** Dioxus hook for loading async data reactively
**When:** Database queries in components, auto-reload on signal changes
**Why:** Handles loading/error states, integrates with Signal reactivity

**Example:**
```rust
let exercises = use_resource(move || {
    let db = state.database().unwrap();
    async move { db.list_exercises().await }
});

match exercises.read_unchecked() {
    Some(Ok(data)) => rsx! { /* render data */ },
    Some(Err(e)) => rsx! { /* error UI */ },
    None => rsx! { /* loading spinner */ }
}
```

### Pattern 3: LEFT JOIN with MAX for "Last Performed"
**What:** SQL query to get most recent session per exercise
**When:** Displaying exercise metadata (last workout date)
**Why:** Efficient aggregate query, single query per exercise

**Example:**
```sql
SELECT
    e.name,
    MAX(s.completed_at) as last_performed,
    COUNT(DISTINCT s.id) as total_sessions
FROM exercises e
LEFT JOIN sessions s ON e.name = s.exercise_name AND s.completed_at IS NOT NULL
WHERE e.name = ?
GROUP BY e.name
```

**Source:** [SQLite JOIN Tutorial](https://www.sqlitetutorial.net/sqlite-join/), [SQLite Aggregate Functions](https://www.sqlitetutorial.net/sqlite-aggregate-functions/)

### Pattern 4: Shared Context via use_context
**What:** Existing pattern in codebase for WorkoutState
**When:** Passing app state to deeply nested components
**Why:** Avoids prop drilling, single source of truth

**Example:**
```rust
// In App component
let workout_state = use_context_provider(WorkoutState::new);

// In child components
fn ExerciseLibraryTab(state: WorkoutState) -> Element {
    let db = state.database(); // Access shared state
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Router for Simple Tabs
**What:** Using dioxus-router for 2-view tab navigation
**Why bad:** Adds complexity (URL routing, back button handling), overkill for simple tabs
**Instead:** Use Signal<Tab> with conditional rendering (see Pattern 1)

### Anti-Pattern 2: N+1 Query Problem
**What:** Querying exercise stats inside loop without batching
**Why bad:** 100 exercises = 100 database queries, slow rendering
**Detection:** `use_resource` inside `.map()` or `for` loop
**Prevention:**
- Phase 1: Acceptable for MVP (<20 exercises expected)
- Phase 2: Batch query if >50 exercises detected
```rust
// BAD (but acceptable for MVP)
for exercise in exercises {
    let stats = use_resource(|| db.get_exercise_stats(&exercise.name));
}

// GOOD (future optimization)
let all_stats = use_resource(|| db.get_all_exercise_stats());
```

### Anti-Pattern 3: Blocking Database Calls in Render
**What:** Calling `db.list_exercises().await` directly in component body
**Why bad:** Blocks rendering, no loading state, breaks Dioxus reactivity
**Instead:** Always use `use_resource` for async database queries

```rust
// BAD
fn ExerciseLibraryTab(state: WorkoutState) -> Element {
    let exercises = state.database().unwrap().list_exercises().await; // ERROR: can't await in sync fn
}

// GOOD
fn ExerciseLibraryTab(state: WorkoutState) -> Element {
    let exercises = use_resource(move || async move {
        state.database().unwrap().list_exercises().await
    });
}
```

### Anti-Pattern 4: Tab State in Multiple Places
**What:** Storing `active_tab` separately from WorkoutState
**Why bad:** State divergence, hard to synchronize, violates single source of truth
**Instead:** Add `active_tab: Signal<Tab>` to WorkoutState, access via context

## Scalability Considerations

| Concern | At 10 exercises | At 50 exercises | At 200 exercises |
|---------|----------------|-----------------|------------------|
| **List rendering** | Instant | Instant | Instant (list rendering fast) |
| **Stats loading** | 10 queries, ~100ms | 50 queries, ~500ms | Batch query needed (see Anti-Pattern 2) |
| **Search filtering** | Client-side fine | Client-side fine | Client-side fine (JS array filter fast) |
| **Memory usage** | Negligible | <1MB | <5MB (SQLite efficient) |

**Optimization Trigger:** If exercise count >100, implement batch stats query:
```rust
pub async fn get_all_exercise_stats(&self) -> Result<HashMap<String, ExerciseStats>, DatabaseError> {
    let sql = r#"
        SELECT
            e.name,
            MAX(s.completed_at) as last_performed,
            COUNT(DISTINCT s.id) as total_sessions
        FROM exercises e
        LEFT JOIN sessions s ON e.name = s.exercise_name AND s.completed_at IS NOT NULL
        GROUP BY e.name
    "#;
}
```

## Database Schema Changes

### Existing Schema (v1.0)
```sql
CREATE TABLE exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    is_weighted INTEGER NOT NULL,
    min_weight REAL,
    increment REAL
);

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_name TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER
);
```

**Analysis:** Schema already supports exercise library queries via:
- `exercises` table contains all exercise metadata
- `sessions.exercise_name` foreign key enables stats aggregation
- No schema changes needed for Phase 1-2

### Phase 3: Archive Support (NEW column)
```sql
-- Migration query
ALTER TABLE exercises ADD COLUMN archived_at INTEGER DEFAULT NULL;

-- Query for active exercises
SELECT * FROM exercises WHERE archived_at IS NULL;
```

**Rationale:** Soft delete preserves historical data. Sessions table references `exercise_name` (text), so archiving doesn't break foreign key constraints.

## Build Order (Dependency-Aware)

### Phase 1: Foundation (2-3 hours)
**Goal:** Tab navigation working, no library functionality yet

1. **Add Tab enum and active_tab to WorkoutState** (30 min)
   - Define `Tab` enum
   - Add `active_tab: Signal<Tab>` field
   - Add getter/setter methods
   - Default to `Tab::Workout`

2. **Create MainInterface and TabBar components** (1 hour)
   - MainInterface wraps existing WorkoutInterface
   - TabBar renders buttons, sets active_tab
   - Match expression for conditional rendering
   - **Test:** Clicking tabs switches views

3. **Rename WorkoutInterface to WorkoutTab** (30 min)
   - Refactor existing component
   - Update App.rs to use MainInterface
   - **Test:** Workout functionality unchanged

4. **Create stub ExerciseLibraryTab** (1 hour)
   - Placeholder component with "Coming soon" text
   - **Test:** Tab switching works end-to-end

**Validation:** User can switch between tabs, workout session persists across switches.

### Phase 2: Exercise List (3-4 hours)
**Goal:** Display exercise list with search

**Dependencies:** Phase 1 complete

1. **Add Database.list_exercises() method** (1 hour)
   - SQL query to `exercises` table
   - Map results to `Vec<ExerciseMetadata>`
   - Handle empty table case
   - **Test:** Unit test with sample data

2. **Implement ExerciseLibraryTab with use_resource** (1.5 hours)
   - use_resource hook for async query
   - Loading/error/success states
   - Map to ExerciseCard components
   - **Test:** Exercises display in UI

3. **Create ExerciseCard component** (1 hour)
   - Display exercise name and type
   - Placeholder for stats (show "No data")
   - DaisyUI card styling
   - **Test:** Cards render correctly

4. **Add search input and client-side filtering** (30 min)
   - use_signal for search_query
   - Filter exercises by name (case-insensitive)
   - **Test:** Search filters list

**Validation:** User can view all exercises, search by name.

### Phase 3: Exercise Metadata (2-3 hours)
**Goal:** Show last performed date and session count

**Dependencies:** Phase 2 complete

1. **Add Database.get_exercise_stats() method** (1.5 hours)
   - LEFT JOIN query with MAX aggregate
   - Map to ExerciseStats struct
   - Handle exercises never performed
   - **Test:** Unit test with sample sessions

2. **Update ExerciseCard with stats use_resource** (1 hour)
   - Fetch stats per exercise
   - Display last performed date (formatted)
   - Display total sessions count
   - Loading state while fetching
   - **Test:** Stats display correctly

3. **Format dates for display** (30 min)
   - Convert Unix timestamp to "X days ago" or date
   - Handle None (never performed) case
   - **Test:** Date formatting edge cases

**Validation:** Each exercise shows when last performed and total session count.

### Phase 4: Edit Exercise (Deferred - Complex)
**Goal:** Allow editing exercise name and config

**Dependencies:** Phase 3 complete
**Complexity:** Modal UI, validation, database update with name change

**Recommendation:** Defer to separate milestone or Phase 4+ due to:
- Name change affects `sessions.exercise_name` references
- Requires transaction to update both tables
- Validation complexity (prevent duplicate names)
- Modal component infrastructure

**Alternative for v1.1:** Read-only library, edit via "Start Session" flow (existing)

### Phase 5: Archive Exercise (Deferred - Schema Change)
**Goal:** Soft delete exercises

**Dependencies:** Phase 3 complete
**Complexity:** Database migration, filtered queries

**Recommendation:** Defer to v1.2+ due to:
- Schema migration required (add `archived_at` column)
- All list queries need `WHERE archived_at IS NULL`
- Unarchive functionality needed for mistakes
- Archive UI placement (requires ExerciseCard actions)

## Build Order Summary

**Recommended for v1.1:**
1. Phase 1: Foundation (tabs working)
2. Phase 2: Exercise List (view exercises, search)
3. Phase 3: Exercise Metadata (last performed, session count)

**Defer to future milestones:**
- Phase 4: Edit Exercise (complex, name change implications)
- Phase 5: Archive Exercise (schema migration, filtered queries)

**Total Estimated Effort:** 7-10 hours for Phases 1-3

## Integration Testing Strategy

### Test 1: Tab Navigation with Active Session
```
Given: User has active workout session (ActiveSession view)
When: User clicks "Library" tab
Then: Library view displays, session persists in state
When: User clicks "Workout" tab
Then: ActiveSession view restores with same session data
```

### Test 2: Exercise List After Session
```
Given: User completes workout session for "Bench Press"
When: User navigates to Library tab
Then: "Bench Press" appears in exercise list
And: Last performed shows today's date
And: Total sessions shows 1
```

### Test 3: Search Filter
```
Given: Library contains "Bench Press", "Squat", "Deadlift"
When: User types "press" in search
Then: Only "Bench Press" displays
When: User clears search
Then: All 3 exercises display
```

### Test 4: Empty State
```
Given: New database with no exercises
When: User navigates to Library tab
Then: "No exercises yet" message displays
And: User can navigate back to Workout tab to create first exercise
```

## Sources

- [Dioxus 0.7 Navigation Documentation](https://dioxuslabs.com/learn/0.7/essentials/router/navigation/)
- [Dioxus Signals State Management](https://dioxuslabs.com/learn/0.7/essentials/basics/signals/)
- [Dioxus use_signal Tutorial](https://medium.com/@mikecode/dioxus-10-state-use-signal-79b3c20b6b59)
- [SQLite JOIN Tutorial](https://www.sqlitetutorial.net/sqlite-join/)
- [SQLite Aggregate Functions](https://www.sqlitetutorial.net/sqlite-aggregate-functions/)
- [SQLite Built-in Aggregate Functions](https://sqlite.org/lang_aggfunc.html)
- [SQLite Group By](https://www.sqlitetutorial.net/sqlite-group-by/)
