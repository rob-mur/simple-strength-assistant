# Pitfalls Research

**Domain:** Exercise Library for Workout Tracking PWA
**Researched:** 2026-03-01
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Archive Without Foreign Key Enforcement

**What goes wrong:**
Archiving exercises without proper foreign key constraints enabled leads to orphaned workout data. Users archive "Bench Press" but historical sessions still reference it by name, creating inconsistent views where workout history displays archived exercises or breaks entirely.

**Why it happens:**
SQLite has foreign keys DISABLED by default for backward compatibility. Developers assume constraints are enforced automatically, but without `PRAGMA foreign_keys = ON;` in every database session, CASCADE behaviors and referential integrity are silently ignored.

**How to avoid:**
- Execute `PRAGMA foreign_keys = ON;` immediately after opening database connection (every session)
- Add an `is_archived` boolean column to exercises table instead of deleting rows
- Create database constraint tests that verify FK enforcement is active
- Index foreign key columns for 20% performance improvement in join operations

**Warning signs:**
- Archived exercises still appear in workout history
- No error when attempting to delete exercises with dependent sets
- Tests pass but production has orphaned records
- Query plans show full table scans on session_id lookups

**Phase to address:**
Phase 1 (Exercise Library Foundation) - Must be correct before any archive UI is built

---

### Pitfall 2: Case-Insensitive Search Without Proper Indexing

**What goes wrong:**
Implementing case-insensitive exercise search using `LOWER(name) LIKE ?` without indexed support causes 2-5x performance degradation as exercise count grows. Search becomes sluggish with 100+ exercises, defeating the purpose of search functionality.

**Why it happens:**
Developers add `WHERE LOWER(name) LIKE LOWER(?)` for case-insensitive matching, but SQLite indexes don't help with function-wrapped columns. The database performs full table scans on every keystroke, compounded by reactive UI triggering queries without debouncing.

**How to avoid:**
- Use SQLite's `COLLATE NOCASE` on the name column definition: `name TEXT NOT NULL UNIQUE COLLATE NOCASE`
- Create index with NOCASE collation: `CREATE INDEX idx_exercise_name ON exercises(name COLLATE NOCASE)`
- Implement 300-500ms debounce on search input to prevent query spam
- Use `.peek()` on Dioxus signals in search handlers to avoid unnecessary subscriptions
- Consider normalized search column if handling international characters (SQLite NOCASE is ASCII-only)

**Warning signs:**
- Search feels slow with modest exercise counts (50-100 items)
- Browser DevTools shows queries every keystroke
- Query execution time increases linearly with exercise count
- EXPLAIN QUERY PLAN shows "SCAN TABLE exercises" instead of index usage

**Phase to address:**
Phase 2 (Search & Filter) - Search UX must be responsive from the start

---

### Pitfall 3: Reactive Signal Cascades on Every Keystroke

**What goes wrong:**
Connecting search input directly to reactive signals without debouncing triggers full re-renders, database queries, and signal updates on every keystroke. This creates 13x more signal effects than necessary, causing UI lag, battery drain, and poor mobile UX.

**Why it happens:**
Dioxus signals make reactivity easy, leading developers to wire input → signal → query → UI update directly. Each keystroke writes to a signal, which triggers query execution, which updates results signal, which re-renders components. The "it works" demo hides the performance cost until production use.

**How to avoid:**
- Debounce search input before writing to signals (300-500ms delay)
- Use `.peek()` for reading signals in event handlers without subscribing
- Separate "input text" signal from "search query" signal with debounced sync
- Batch related signal updates to prevent cascade re-renders
- Avoid signal updates in tight loops or during scroll events

**Warning signs:**
- Mobile devices feel warm during search
- Input field feels laggy or loses focus
- Battery drains faster when using search
- Signal effects log shows 10+ updates per keystroke
- Error boundary issues where reactive errors don't disappear after valid input

**Phase to address:**
Phase 2 (Search & Filter) - Implement debouncing before exposing to users

---

### Pitfall 4: Exercise Rename Breaking Workout History References

**What goes wrong:**
Allowing users to rename exercises without migrating historical session data creates broken references. User renames "Squat" to "Back Squat" but sessions table still references "Squat" by name, making historical data invisible or causing query failures.

**Why it happens:**
The current schema stores `exercise_name TEXT` in sessions table instead of `exercise_id INTEGER`. This denormalization optimizes for simple queries but prevents safe renames. Developers don't realize the data integrity constraint until users report missing workout history.

**How to avoid:**
- Use `exercise_id` foreign key in sessions table instead of storing name directly
- JOIN exercises table when displaying session data to get current name
- If keeping denormalized name for performance, wrap renames in transaction:
  1. Update exercises.name
  2. Update all sessions.exercise_name WHERE exercise_name = old_name
  3. Update all future references in single atomic transaction
- Test rename with existing workout history in automated tests

**Warning signs:**
- User feedback about "disappeared" workout history after editing exercise
- Duplicate exercises appearing in autocomplete (old name + new name)
- Exercise statistics don't match between library view and history view
- No SQL transaction wrapping rename operations

**Phase to address:**
Phase 3 (Edit Exercise) - Critical to get right before allowing edits

---

### Pitfall 5: No Empty State for First-Time Users

**What goes wrong:**
Launching directly into a blank exercise list confuses new users. They see an empty screen with search box and no guidance, don't understand they need to add exercises first, and abandon the app thinking it's broken.

**Why it happens:**
Developers focus on the "working state" (list with exercises) and overlook the first-run experience. Empty states are dismissed as "just showing a message" rather than critical onboarding.

**How to avoid:**
- Detect empty exercise list and show purpose-built empty state UI
- Include clear "Add Your First Exercise" call-to-action button
- Consider pre-loading 5-10 common exercises (Squat, Bench Press, Deadlift, etc.) as starter content
- Show context: "Your exercise library is empty. Add exercises to start tracking workouts."
- Design empty state before building main UI to ensure it's not an afterthought

**Warning signs:**
- Blank white screen on first launch
- Only placeholder text like "No exercises found"
- No clear action for new users
- User testing shows confusion about "what to do first"
- High bounce rate on exercise library tab for new installs

**Phase to address:**
Phase 1 (Exercise Library Foundation) - First impression matters for retention

---

### Pitfall 6: Partial Updates from Failed Transactions

**What goes wrong:**
Browser crash or power loss mid-transaction leaves database in partial state. User edits exercise name and archived status, but only name updates commit. Database shows exercise as active with new name, breaking data consistency.

**Why it happens:**
sql.js runs entirely in-memory; changes only persist when database is explicitly exported to file. If browser crashes between updating exercise and exporting, some changes are lost. Developers treat sql.js like server SQLite without understanding WASM persistence model.

**How to avoid:**
- Wrap all multi-field updates in explicit transactions: `BEGIN; UPDATE...; UPDATE...; COMMIT;`
- Test transaction rollback: simulate crashes during multi-step operations
- Enable WAL mode for better crash recovery: `PRAGMA journal_mode=WAL`
- Export database to file system immediately after transaction commit
- Implement auto-save with debounced writes to minimize data loss window
- Show user clear saving state indicators ("Saving...", "Saved", "Error")

**Warning signs:**
- Inconsistent exercise data after browser crashes
- Some fields update while others don't
- No transaction boundaries in edit code
- Export to file only on manual save, not automatic
- Missing error recovery UI for failed saves

**Phase to address:**
Phase 3 (Edit Exercise) - Transaction safety before exposing edits

---

### Pitfall 7: Archive UI That Feels Like Permanent Delete

**What goes wrong:**
Users accidentally archive exercises thinking they're just hiding them temporarily, then panic when they can't find them. Or conversely, users think "archive" means permanent delete and hesitate to use it, defeating the feature's purpose.

**Why it happens:**
Archive behavior varies wildly across platforms. Instagram archives hide from profile but can be restored. Email archives move to folder. GitHub archives make repos read-only. Without clear UI communication, users project their own mental model onto the feature.

**How to avoid:**
- Label clearly: "Archive (hide from library)" not just "Archive"
- Show undo notification immediately after archiving: "Exercise archived. Undo?"
- Provide obvious "Show archived exercises" toggle in UI
- Display archived count when hidden: "5 archived exercises (show)"
- Make unarchive trivially easy: single tap/click, no confirmation needed
- Use non-destructive language: avoid red colors/delete icons for archive action
- Include "What happens when I archive?" help text near feature

**Warning signs:**
- User testing shows confusion about archive behavior
- Support requests about "recovering deleted exercises"
- Archive button uses trash can icon (implies deletion)
- No easy way to view/restore archived items
- Archive action requires multiple confirmations

**Phase to address:**
Phase 4 (Archive Exercise) - UX clarity prevents user anxiety

---

### Pitfall 8: Duplicate Exercises from Import/Restore

**What goes wrong:**
Restoring from backup or importing exercises creates duplicates when uniqueness constraint only checks name. User has "Bench Press (archived)" and "Bench Press (active)" as separate database rows, breaking workout tracking and confusing exercise selection.

**Why it happens:**
Database has `UNIQUE` constraint on exercise name but not on (name, is_archived) tuple. When archiving, name stays same but is_archived flips. Restoring backup re-inserts archived version, violating no constraint. Or worse, developer removes archive flag to "restore" by inserting new row.

**How to avoid:**
- Archive is status change, not new row: `UPDATE exercises SET is_archived = 1 WHERE id = ?`
- Unarchive is status change: `UPDATE exercises SET is_archived = 0 WHERE id = ?`
- Never allow duplicate names across archived and active exercises
- Unique constraint should be on name alone: `name TEXT NOT NULL UNIQUE`
- Provide merge tool if duplicates exist: "Found duplicate 'Bench Press'. Which to keep?"
- Validate on import: check for conflicts before inserting

**Warning signs:**
- Exercise appears twice in autocomplete with same name
- Workout history shows same exercise multiple times with different IDs
- Search returns duplicate results
- Database allows INSERT of same name when one is archived

**Phase to address:**
Phase 4 (Archive Exercise) - Prevent duplicates, don't patch later

---

### Pitfall 9: Memory Bloat from sql.js In-Memory Database

**What goes wrong:**
Database grows beyond browser's WASM memory limit (2-4GB depending on browser). With 2 years of workout data, high-frequency tracking, and exercise history, database reaches hundreds of MB. sql.js loads entire database into memory, causing "Out of Memory" crashes on mobile browsers.

**Why it happens:**
sql.js operates entirely in-memory for performance. Every database operation requires full database in RAM. Developers test with small datasets (few exercises, dozen sessions) but production users accumulate thousands of sets over months. Mobile browsers have tighter memory constraints than desktop.

**How to avoid:**
- Monitor database file size in UI: show warning at 50MB, 100MB thresholds
- Implement data retention policy: archive sessions older than 2 years
- Paginate large queries: don't load entire workout history at once
- Test with realistic data volume: 500+ exercises, 1000+ sessions, 10,000+ sets
- Consider migration to OPFS (Origin Private File System) persistence for larger datasets
- Use indexed subset queries instead of full table scans
- Compress exported database files

**Warning signs:**
- Mobile browser crashes during database operations
- Memory usage increases linearly with database size
- "Out of Memory" errors in console
- Slow initial load times on older devices
- Database export hangs or fails

**Phase to address:**
Phase 1 (Exercise Library Foundation) - Design with scalability in mind

---

### Pitfall 10: Search Matches Archived Exercises

**What goes wrong:**
Search returns archived exercises mixed with active ones, confusing users. User archives "Old Bench Press" variation, searches "Bench", gets both archived and active versions. Selecting archived exercise breaks workout flow.

**Why it happens:**
Search query filters by name but forgets to check `is_archived` flag. Developer tests search with all-active exercises, ships feature, and users report seeing "deleted" items in results.

**How to avoid:**
- Default search filters to `WHERE is_archived = 0 AND name LIKE ?`
- Provide explicit "Include archived" toggle for power users
- Show archived status visually in search results if included (grayed out, badge)
- Separate "Browse all exercises" from "Select exercise for workout" UX
- Test search with mix of archived and active exercises
- Document filter behavior clearly

**Warning signs:**
- User feedback about "deleted exercises still appearing"
- No is_archived filter in search SQL
- Same search UI used for browsing and workout exercise selection
- No visual distinction between archived and active in results

**Phase to address:**
Phase 2 (Search & Filter) - Prevent confusion from start

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Store exercise_name instead of exercise_id in sessions | Simpler queries, no joins needed | Rename breaks history, duplicate cleanup hard | Never - schema migration now cheaper than later |
| Skip debouncing search input | Faster development, fewer dependencies | Poor mobile performance, battery drain | Only in prototype/MVP with warning comment |
| No transaction wrapping for edits | Less code, simpler error handling | Partial updates on crashes, data corruption | Never - transactions are critical |
| Hard-delete instead of archive | Simpler schema, no filtered queries | Orphaned session data, user regret | Never for user-created content |
| PRAGMA foreign_keys only in init code | Works in same session | Breaks after reconnect, silent FK violations | Never - must be per-session |
| Skip empty state UI | Ship faster, focus on "real" features | Poor first-run experience, user confusion | Early prototype only, not production |
| Load entire exercise list without pagination | Simpler code, works for small datasets | Memory issues at scale, slow rendering | Acceptable up to ~500 exercises, then paginate |
| Use LIKE without COLLATE NOCASE index | Search "works" immediately | Slow search with 100+ exercises | Acceptable for <50 exercises in testing phase |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| File System Access API | Assuming writes are atomic | Use createWritable() for atomic writes via temporary file |
| sql.js transactions | Treating like server-side SQLite | Export to file after COMMIT to persist changes |
| Dioxus signals | Updating in event handlers without peek() | Use .peek() for reads to avoid unwanted subscriptions |
| Foreign key constraints | Assuming they're enabled by default | Execute PRAGMA foreign_keys = ON every session |
| WASM memory | Assuming unlimited memory like server | Monitor size, test on mobile, implement retention |
| LocalStorage fallback | Treating as equivalent to File System API | Document size limits (5-10MB), warn users |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Unindexed foreign keys | Slow joins, full table scans | Index session_id, exercise_id columns | 1000+ sessions |
| Search without debouncing | UI lag, excessive queries | 300-500ms debounce on input | Any keystroke input |
| Reactive cascade updates | Mobile heat, battery drain | Use .peek(), batch updates | Moderate signal usage |
| Loading full workout history | Slow initial render | Paginate, lazy load | 100+ sessions |
| Case functions in WHERE | Slow search, full scans | COLLATE NOCASE + index | 100+ exercises |
| Full database exports on each save | UI freeze during save | Debounced auto-save (1-5 sec) | Database >10MB |
| Memory growth from unused signals | Gradual slowdown | Clean up signals in useEffect | Long sessions |
| WASM-to-JS boundary crossing | Slow bulk operations | Batch queries, minimize calls | High-frequency operations |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| No input sanitization on exercise names | SQL injection via name field | Use parameterized queries, validate input length |
| Allowing arbitrary SQL in search | Injection attacks | Use prepared statements, whitelist operations |
| Storing sensitive user data without encryption | Data exposure if file shared | Don't store PII; workout data is semi-public |
| No validation on weight/rep inputs | Negative values, NaN, Infinity in database | Validate min/max bounds, type check before insert |
| Trusting file contents on restore | Malicious SQLite database code execution | Validate schema, sanitize on import |
| Missing CORS headers for database file | Can't load from other origins | Not applicable - local file system only |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No visual feedback on archive | User unsure if action worked | Show toast: "Exercise archived. Undo?" |
| Search shows archived by default | Confusion about "deleted" items | Filter archived out, add "Show archived" toggle |
| Rename without history update | Lost workout history | Transaction: update name + migrate references |
| No empty state guidance | New users don't know what to do | Clear CTA: "Add your first exercise" |
| Archive uses delete icon | Users think it's permanent | Use archive box icon, label clearly |
| No undo on accidental archive | User anxiety, hesitant usage | Immediate undo toast, easy unarchive |
| Hidden archived count | Users forget archived items exist | Show "5 archived exercises (show)" |
| Same UI for browse vs. workout selection | Wrong context for archived exercises | Separate flows, different filters |
| No save status indicator | Users unsure if edits persisted | Show "Saving...", "Saved", "Error" states |
| Long exercise lists without virtual scrolling | Laggy scroll on mobile | Virtual scroll for 100+ items |

## "Looks Done But Isn't" Checklist

- [ ] **Foreign key enforcement:** Often missing `PRAGMA foreign_keys = ON` in session init — verify with constraint violation test
- [ ] **Search indexing:** Often missing COLLATE NOCASE index — verify with EXPLAIN QUERY PLAN
- [ ] **Input debouncing:** Often missing debounce on search — verify signal effect count in logs
- [ ] **Transaction boundaries:** Often missing BEGIN/COMMIT around multi-step edits — verify rollback test
- [ ] **Archive filtering:** Often missing is_archived = 0 in search — verify with archived exercise in test data
- [ ] **Empty state UI:** Often missing first-run experience — verify with fresh database
- [ ] **Save error handling:** Often missing error UI for failed exports — verify with simulated write failure
- [ ] **Memory monitoring:** Often missing database size tracking — verify with large dataset test
- [ ] **Duplicate prevention:** Often missing uniqueness validation on restore — verify with import test
- [ ] **Rename migration:** Often missing session data update — verify with historical workout data

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Missing FK enforcement | LOW | Add PRAGMA to init, test constraints, no migration needed |
| Orphaned session data | MEDIUM | Write cleanup script, manual merge for ambiguous cases |
| Broken rename history | HIGH | Manual data recovery: match sessions to exercises by timestamp, context |
| Memory bloat | MEDIUM | Implement retention policy, provide export/delete old data tool |
| Duplicate exercises | MEDIUM | Build merge tool: user picks which to keep, migrate references |
| Slow search | LOW | Add index with migration, immediate improvement |
| No transactions | HIGH | Fix new code, historical data may have inconsistencies |
| Archive UX confusion | LOW | Update UI labels, add help text, no data fix needed |
| Missing debounce | LOW | Add debounce to signal updates, immediate improvement |
| Poor empty state | LOW | Build empty state component, deploy in next release |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Archive without FK enforcement | Phase 1 - Foundation | Test foreign key violation throws error |
| Case-insensitive search indexing | Phase 2 - Search & Filter | EXPLAIN QUERY PLAN shows index usage |
| Reactive signal cascades | Phase 2 - Search & Filter | Log signal effects, count should be <5 per keystroke |
| Rename breaking history | Phase 3 - Edit Exercise | Test rename with existing sessions, verify history intact |
| No empty state | Phase 1 - Foundation | Load with empty database, verify CTA visible |
| Partial update failures | Phase 3 - Edit Exercise | Simulate crash mid-transaction, verify rollback |
| Archive UI confusion | Phase 4 - Archive | User testing shows 80%+ understand archive behavior |
| Duplicate exercises | Phase 4 - Archive | Test restore with archived exercises, verify no duplicates |
| Memory bloat | Phase 1 - Foundation | Load test with 10,000+ sets, monitor memory usage |
| Search includes archived | Phase 2 - Search & Filter | Test search with archived exercises, verify filtered |

## Sources

**Database & SQLite:**
- [SQLite Foreign Key Support](https://sqlite.org/foreignkeys.html) - Foreign key enforcement documentation (HIGH confidence)
- [SQLite CASCADE DELETE: Complete Guide 2026](https://copyprogramming.com/howto/sqlite-cascade-delete) - CASCADE best practices (MEDIUM confidence)
- [SQLite Atomic Commit](https://sqlite.org/atomiccommit.html) - Transaction and rollback mechanisms (HIGH confidence)
- [SQLite Database Corruption Prevention](https://runebook.dev/en/articles/sqlite/howtocorrupt) - How corruption happens and prevention (HIGH confidence)
- [Best Practices for Data Consistency in SQLite Transactions](https://www.slingacademy.com/article/best-practices-for-data-consistency-in-sqlite-transactions/) - Transaction patterns (MEDIUM confidence)
- [Common Pitfalls in SQLite Transactions](https://www.slingacademy.com/article/common-pitfalls-in-sqlite-transactions-and-how-to-avoid-them/) - Transaction mistakes (MEDIUM confidence)

**WASM & Performance:**
- [sql.js GitHub Repository](https://github.com/sql-js/sql.js) - Memory limitations and Asyncify tradeoffs (HIGH confidence)
- [sql.js Out of Memory Issues](https://github.com/sql-js/sql.js/issues/574) - Memory leak investigation (HIGH confidence)
- [The State of WebAssembly 2025-2026](https://platform.uno/blog/the-state-of-webassembly-2025-2026/) - WASM performance context (MEDIUM confidence)
- [Notion's WASM SQLite Performance](https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite) - Real-world WASM optimization (HIGH confidence)
- [SQLite Persistence on Web - November 2025](https://www.powersync.com/blog/sqlite-persistence-on-the-web) - Current state of browser persistence (HIGH confidence)

**Reactivity & State Management:**
- [Dioxus Reactivity Documentation](https://deepwiki.com/DioxusLabs/dioxus/3-reactivity-and-state-management) - Signal patterns (HIGH confidence)
- [Dioxus Signal Issues](https://github.com/DioxusLabs/dioxus/issues/4039) - ErrorBoundary reactivity bugs (HIGH confidence)
- [React Debouncing Guide](https://www.developerway.com/posts/debouncing-in-react) - Debounce patterns (MEDIUM confidence)
- [Angular Signals & Debouncing 2026](https://dev.to/cristiansifuentes/angular-signals-debouncing-a-scientific-production-minded-guide-2026-2kdk) - Modern reactivity patterns (MEDIUM confidence)

**UX & Design:**
- [Exercise Library UX Mistakes](https://www.agiratech.com/top-5-common-ux-ui-mistakes-made-in-fitness-apps-that-you-know) - Fitness app UX research (MEDIUM confidence)
- [Empty State UX Best Practices](https://www.eleken.co/blog-posts/empty-state-ux) - Empty state design patterns (HIGH confidence)
- [Empty States in User Onboarding](https://www.useronboard.com/onboarding-ux-patterns/empty-states/) - First-time user experience (HIGH confidence)
- [Archive/Unarchive Implementation](https://socialrails.com/blog/how-to-archive-unarchive-instagram-posts) - User expectations for archive features (MEDIUM confidence)
- [Fitness App Development Challenges](https://codiant.com/blog/challenges-in-building-a-fitness-app/) - Domain-specific pitfalls (MEDIUM confidence)

**Data Integrity:**
- [OWASP Input Validation Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html) - Security validation patterns (HIGH confidence)
- [Input Validation and Sanitization 2025](https://wnesecurity.com/input-validation-and-sanitization-2024-how-to-do-it/) - Current best practices (MEDIUM confidence)
- [Duplicate Exercise Handling](https://help.strongapp.io/article/209-how-do-i-merge-data-between-two-exercises) - Real-world merge strategies (HIGH confidence)
- [File System Access API Security](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API) - API constraints and risks (HIGH confidence)

**Workout Tracking Domain:**
- [Soft Delete Archive Patterns](https://medium.com/meroxa/creating-a-soft-delete-archive-table-with-postgresql-70ba2eb6baf3) - Archive table design (MEDIUM confidence)
- [Avoiding Soft Delete Anti-Pattern](https://news.ycombinator.com/item?id=40326815) - Community discussion on soft delete tradeoffs (LOW confidence)
- [2026 Fitness Trends](https://www.feed.fm/2026-digital-fitness-ecosystem-report) - User expectations for fitness apps (MEDIUM confidence)

---
*Pitfalls research for: Exercise Library feature in workout tracking PWA*
*Researched: 2026-03-01*
