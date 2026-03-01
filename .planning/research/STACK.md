# Stack Research: Exercise Library

**Domain:** Exercise management UI for existing workout tracking PWA
**Researched:** 2026-03-01
**Confidence:** HIGH

## Executive Summary

The Exercise Library milestone requires **ZERO new dependencies**. All required capabilities already exist in the validated v1.0 stack:

- **Search/Filter:** Client-side filtering with Dioxus reactive signals (no library needed)
- **Edit Forms:** Dioxus form patterns already proven in StartSessionView
- **Archive:** SQLite column addition (no migration library needed - simple ALTER TABLE)
- **UI Components:** DaisyUI components already used throughout app
- **State Management:** Dioxus 0.7.2 Signals already handle reactive lists

The only additions needed are:
1. New database queries for list/update/archive operations
2. New Dioxus components using existing patterns
3. One new column in existing `exercises` table

## Stack Status: No Changes Required

### Existing Stack (Validated v1.0)

| Technology | Version | Already Handles | Notes |
|------------|---------|-----------------|-------|
| **Dioxus** | 0.7.2 | Reactive lists, forms, search filtering | Native `for` loops in `rsx!` macro, `use_signal` for state |
| **SQLite (sql.js)** | 1.13.0 | Full CRUD, ALTER TABLE, LIKE queries | Latest version with SQLite 3.49 core |
| **DaisyUI** | 4.12.14 | Cards, inputs, buttons, badges, tables | Already used in ActiveSession, StartSessionView |
| **Tailwind CSS** | 3.4.17 | Styling, responsive layout | Zero-setup with Dioxus 0.7 |

### What Each Feature Needs (From Existing Stack)

| Feature | Implementation | Existing Capability |
|---------|----------------|---------------------|
| **Exercise List View** | Query all exercises, map to DaisyUI cards | `db.execute()` + `for` loop in `rsx!` |
| **Search** | Filter exercises array on name field | `use_signal` + `.filter()` + client-side string matching |
| **Edit Exercise** | Update form with validation (like StartSessionView) | Existing form patterns, `save_exercise()` already exists |
| **Archive** | Add `archived_at` column, filter WHERE archived_at IS NULL | ALTER TABLE (one-time), standard SQL query |
| **Metadata Display** | Query sessions/sets tables, aggregate counts | SQL JOIN + GROUP BY (already using similar patterns) |

## Database Changes Required

### Schema Addition (Not a Stack Change)

```sql
-- Add to exercises table (one-time migration in db.rs create_tables)
ALTER TABLE exercises ADD COLUMN archived_at INTEGER DEFAULT NULL;

-- Create index for faster filtering
CREATE INDEX IF NOT EXISTS idx_exercises_archived ON exercises(archived_at);
```

**Why no migration library needed:**
- SQLite `ALTER TABLE ADD COLUMN` is idempotent with `IF NOT EXISTS` pattern
- Single-direction migration (no rollback needed for MVP)
- Existing `create_tables()` function already runs on every init
- Can check column existence with `PRAGMA table_info(exercises)` if needed

## Required Query Additions

These are **code additions**, not dependency additions. All use existing `Database::execute()`:

```rust
// 1. List all non-archived exercises
pub async fn list_exercises(&self) -> Result<Vec<ExerciseRow>, DatabaseError> {
    let sql = "SELECT * FROM exercises WHERE archived_at IS NULL ORDER BY name ASC";
    // ... existing execute() pattern
}

// 2. Search exercises by name
pub async fn search_exercises(&self, query: &str) -> Result<Vec<ExerciseRow>, DatabaseError> {
    let sql = "SELECT * FROM exercises WHERE archived_at IS NULL AND name LIKE ? ORDER BY name ASC";
    // ... existing execute() pattern with params
}

// 3. Get exercise metadata (last performed, session count)
pub async fn get_exercise_stats(&self, exercise_name: &str) -> Result<ExerciseStats, DatabaseError> {
    let sql = r#"
        SELECT
            COUNT(DISTINCT s.id) as session_count,
            MAX(s.completed_at) as last_performed,
            COUNT(cs.id) as total_sets
        FROM exercises e
        LEFT JOIN sessions s ON e.name = s.exercise_name
        LEFT JOIN completed_sets cs ON s.id = cs.session_id
        WHERE e.name = ?
    "#;
    // ... existing execute() pattern
}

// 4. Update exercise details
// Already exists: save_exercise() uses INSERT OR REPLACE

// 5. Archive exercise (soft delete)
pub async fn archive_exercise(&self, exercise_name: &str) -> Result<(), DatabaseError> {
    let sql = "UPDATE exercises SET archived_at = ? WHERE name = ?";
    let now = js_sys::Date::now();
    // ... existing execute() pattern
}
```

**Pattern:** All follow existing `Database::execute()` → `JsValue` conversion pattern already proven in v1.0.

## UI Component Patterns (Existing)

### Dioxus 0.7.2 List Rendering

Already proven in `ActiveSession` history table:

```rust
// Client-side filtering pattern
let mut search_query = use_signal(|| String::new());
let exercises = state.exercises(); // Signal<Vec<Exercise>>

let filtered = exercises()
    .iter()
    .filter(|e| e.name.to_lowercase().contains(&search_query().to_lowercase()))
    .collect::<Vec<_>>();

rsx! {
    // Search input
    input {
        class: "input input-bordered",
        r#type: "text",
        value: "{search_query}",
        oninput: move |e| search_query.set(e.value())
    }

    // List rendering (existing Dioxus 0.7 pattern)
    for exercise in filtered {
        ExerciseCard { exercise: exercise.clone() }
    }
}
```

**Why no search library needed:**
- Small dataset (users have 5-50 exercises, not thousands)
- Client-side `.filter()` is instant for this scale
- Case-insensitive search with `.to_lowercase()` is sufficient
- No fuzzy matching needed (exact substring match is expected UX)

### DaisyUI Components Already Used

From existing codebase:
- **Cards:** `.card`, `.card-body`, `.card-title` (see StartSessionView)
- **Inputs:** `.input`, `.input-bordered` (exercise name input)
- **Buttons:** `.btn`, `.btn-primary`, `.btn-ghost` (session controls)
- **Badges:** `.badge`, `.badge-primary` (set counter in ActiveSession)
- **Tables:** `.table`, `.table-zebra` (history in ActiveSession)
- **Alerts:** `.alert`, `.alert-info` (storage mode banner)

**Pattern for Exercise Library:**
- Exercise list: Cards in grid layout
- Search bar: Input with icon
- Edit modal: Card with form (reuse StartSessionView patterns)
- Archive button: Ghost button with confirmation

## What NOT to Add

| Library/Tool | Why NOT Needed | What to Use Instead |
|--------------|----------------|---------------------|
| **Full-text search library** | Dataset too small (< 100 exercises), simple substring match sufficient | Client-side `.filter()` with `.contains()` |
| **Virtual scrolling library** | Users have 5-50 exercises, not 10,000+ | Native rendering (all items fit in viewport) |
| **Form library** | Dioxus signals + HTML inputs already working | Existing form patterns from StartSessionView |
| **State management library** | Dioxus Signals already handle reactive state | `use_signal`, `use_context_provider` |
| **SQL query builder** | Only 5 new queries, all simple SELECT/UPDATE | Raw SQL strings (existing pattern) |
| **Database migration tool** | Single column addition, no complex migrations | `ALTER TABLE` in `create_tables()` |
| **Virtualized list component** | Small, static dataset | Native `for` loop in `rsx!` |
| **Debounce library** | Search is instant on small dataset | Direct `oninput` handler |

## Version Compatibility

| Package | Current | Compatible With | Notes |
|---------|---------|-----------------|-------|
| dioxus 0.7.2 | ✓ | All existing dependencies | Locked version working well |
| sql.js 1.13.0 | ✓ | SQLite 3.49 features | Supports all needed SQL operations |
| DaisyUI 4.12.14 | ✓ | Tailwind 3.4.17 | Already validated in v1.0 |

**No version bumps needed.** Exercise Library uses existing capabilities only.

## Implementation Checklist

**Rust (src/):**
- [ ] Add `archived_at` column to exercises table in `create_tables()`
- [ ] Add 5 new query methods to `Database` struct (list, search, stats, archive, unarchive)
- [ ] Add `ExerciseListItem` struct for display data (name, stats, config)
- [ ] Add Exercise Library tab component with search input + list view
- [ ] Add ExerciseCard component for list items
- [ ] Add EditExerciseModal component (reuse StartSessionView form patterns)

**No new Cargo.toml dependencies.**
**No new package.json dependencies.**
**No new web-sys features needed** (all DOM APIs already enabled).

## Sources

### Verified Capabilities (HIGH Confidence)

- [Dioxus 0.7 Components Documentation](https://dioxuslabs.com/learn/0.7/essentials/ui/components/) — List rendering with `for` loops
- [Dioxus 0.7.0 Release](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0) — Signal reactivity, hot reloading
- [sql.js GitHub Releases](https://github.com/sql-js/sql.js/releases) — Version 1.13.0 with SQLite 3.49
- [sql.js NPM Package](https://www.npmjs.com/package/sql.js) — Current capabilities confirmed

### Background Research (MEDIUM Confidence)

- [SQLite WASM Full-Text Search Guide](https://blog.ouseful.info/2022/04/06/compiling-full-text-search-fts5-into-sqlite-wasm-build/) — FTS5 capabilities (not needed for this milestone)
- [SQLite WASM Documentation](https://sqlite.org/wasm) — Official WASM APIs (already using via sql.js)

---
*Stack research for: Exercise Library (v1.1)*
*Researched: 2026-03-01*
*Confidence: HIGH - All capabilities verified in existing v1.0 codebase*
