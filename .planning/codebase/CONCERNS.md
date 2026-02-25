# Codebase Concerns

**Analysis Date:** 2026-02-25

## Tech Debt

**RefCell Borrow Failures Silently Handled:**
- Issue: Multiple locations use `try_borrow()` and `try_borrow_mut()` that log errors but continue execution, potentially causing silent state corruption
- Files: `src/state/workout_state.rs`
- Impact: If the `WorkoutState` RefCell is already borrowed, mutations and reads fail silently. Lines 93-97, 101-105, 109-113 log errors but don't propagate them. This could lead to state becoming inconsistent without user awareness.
- Fix approach: Replace RefCell with a better state management pattern (Signal from Dioxus 0.7), or ensure all borrow failures are propagated as user-visible errors rather than just logged

**Input Validation Only in UI Layer:**
- Issue: User input parsing uses `.unwrap_or(0)` defaults rather than validation
- Files: `src/app.rs` lines 351-354
- Impact: Invalid user input (like non-numeric text in number fields) silently defaults to 0, allowing users to log sets with 0 reps or 0 RPE without warning. While validation exists in `src/models/validation.rs`, it's not enforced in the UI parsing layer.
- Fix approach: Add input validation before parsing and show user-facing error messages for invalid inputs rather than silently defaulting to zero values

**Race Condition in Database Initialization:**
- Issue: Concurrent initialization attempts are detected but not queued
- Files: `src/state/workout_state.rs` lines 120-143
- Impact: If `setup_database()` is called multiple times concurrently (e.g., from effect running twice), the second call fails immediately with an error rather than waiting for first to complete. This could cause initialization errors during React-style double-rendering in dev mode.
- Fix approach: Implement a queue or promise cache pattern to ensure concurrent initialization attempts wait for the first attempt to complete rather than failing

**Auto-save Failures Ignored:**
- Issue: Database auto-save after logging a set silently ignores write failures
- Files: `src/state/workout_state.rs` lines 281-292
- Impact: If the auto-save fails (permission denied, disk full, etc.), the set data remains only in memory. If the browser crashes or user navigates away, data is lost. Only logs warning but doesn't notify user.
- Fix approach: Show user notification when auto-save fails, and add a "Save pending" indicator in the UI so users know when data hasn't been persisted

**JavaScript Bridge Layer Has No Rust-Side Error Types:**
- Issue: JS errors are converted to generic string errors, losing structure
- Files: `src/state/db.rs` lines 21-25, `src/state/file_system.rs` lines 53-57
- Impact: Cannot distinguish between different failure modes (permission denied vs file not found vs quota exceeded) in Rust code. Makes error recovery and user messaging less precise.
- Fix approach: Create structured error types that parse JS error messages to identify specific failure modes, enabling targeted error handling and user messaging

## Known Bugs

**Number Input Parsing Allows Invalid Values:**
- Symptoms: User can enter "abc" in weight/reps/RPE fields, which silently becomes 0
- Files: `src/app.rs` lines 351-354
- Trigger: Type non-numeric text into any number input field, then click "Log Set"
- Workaround: None - will create a set with 0 values (which later fails validation but after DB insertion attempt)

**Permission Re-request Loop Potential:**
- Symptoms: File handle permission checking might loop requesting permission
- Files: `public/file-handle-storage.js` lines 62-78
- Trigger: If `requestPermission()` returns 'prompt' instead of 'granted'/'denied', code might not handle it correctly
- Workaround: User must manually grant permission when prompted

## Security Considerations

**LocalStorage Fallback Stores Database in Plain Text:**
- Risk: Fallback storage uses LocalStorage which is readable by any JavaScript on the same origin
- Files: `src/state/file_system.rs` lines 283-288, 336-340
- Current mitigation: Data is only workout logs (not sensitive), but still not encrypted
- Recommendations: Document that data is stored unencrypted. Consider adding an encryption layer if sensitive notes/personal data features are added in the future.

**No Input Sanitization for Exercise Names:**
- Risk: Exercise names are stored directly in SQLite without sanitization
- Files: `src/app.rs` lines 163-174, `src/state/db.rs` lines 209-236
- Current mitigation: Using parameterized queries prevents SQL injection. Max length validation prevents DoS via huge strings.
- Recommendations: Current approach is adequate for local-only app. If data syncing is added, ensure names don't contain XSS vectors if rendered in other contexts.

**File System Access API Permission Persistence:**
- Risk: IndexedDB stores file handle which grants persistent access to user's file
- Files: `public/file-handle-storage.js` lines 24-41
- Current mitigation: Browser's native permission system controls access
- Recommendations: Add UI to revoke permission/clear cached handle. Document that closing browser may require re-granting permission on next open.

## Performance Bottlenecks

**Database Export Copies Entire Database on Every Save:**
- Problem: Each set logged triggers full database export to Uint8Array
- Files: `src/state/workout_state.rs` lines 281-292, `src/state/db.rs` lines 238-250
- Cause: SQL.js in-memory database requires full export to persist changes
- Improvement path: Implement debouncing - only save after 2-3 seconds of inactivity rather than after every set. Or batch multiple sets before saving. Document that rapid logging may cause performance issues on lower-end devices.

**Large File Validation Reads Entire File Into Memory:**
- Problem: File size validation reads entire file even if only checking header
- Files: `src/state/file_system.rs` lines 219-281
- Cause: Reads full file to validate SQLite magic number
- Improvement path: Use streaming read or Blob.slice() to read only first 16 bytes for format validation before reading full file

**JavaScript Array Building in Params:**
- Problem: Query parameters are converted to JS Array one element at a time
- Files: `src/state/db.rs` lines 119-138
- Cause: js_sys::Array::push() is called in a loop
- Improvement path: Pre-allocate array with known size or use more efficient bulk operations

## Fragile Areas

**WASM Boundary Error Handling:**
- Files: `src/state/db.rs` lines 119-250, `src/state/file_system.rs` lines 219-334
- Why fragile: Complex JS interop with many failure points. JsValue errors are opaque. Any change to JS module signatures breaks silently at runtime.
- Safe modification: Always test changes with actual browser. Add integration tests that exercise JS boundary. Consider adding JS-side type checking.
- Test coverage: Gaps in error path testing for JS interop failures

**Initialization State Machine:**
- Files: `src/state/workout_state.rs` lines 120-218
- Why fragile: Complex async state transitions with race condition guards. Easy to introduce deadlocks or invalid state transitions.
- Safe modification: Draw state diagram before modifying. Ensure all transitions are explicitly handled. Add logging at every state transition.
- Test coverage: No tests for concurrent initialization, retry after error, or browser refresh during initialization

**File Handle Permission State Management:**
- Files: `public/file-handle-storage.js` lines 44-84, `src/state/file_system.rs` lines 88-108
- Why fragile: Permission state can change externally (user revokes in browser settings). Cached handles may become invalid at any time.
- Safe modification: Always wrap handle access in try/catch. Gracefully fallback to prompting for new handle if cached handle fails.
- Test coverage: No automated tests for permission revocation or stale handle scenarios

## Scaling Limits

**In-Memory Database Size:**
- Current capacity: Limited by browser's WASM memory allocation (typically 2-4GB max)
- Limit: With 50 bytes per set, could store ~40-80 million sets before hitting memory limits (unrealistic for single user)
- Scaling path: Current approach is adequate for single-user workout tracking. If data grows beyond 100MB, consider migrating to IndexedDB-backed SQL.js or OPFS-based SQLite

**LocalStorage Fallback Storage:**
- Current capacity: LocalStorage typically limited to 5-10MB per origin
- Limit: Could store roughly 50,000-100,000 sets before hitting quota
- Scaling path: Add quota checking and warn user when approaching limits. Provide export/archive functionality for old data.

**Single-File Database:**
- Current capacity: File System Access API files limited by filesystem
- Limit: SQLite file could grow unbounded, potentially multi-GB for long-term users
- Scaling path: Add database maintenance tools (vacuum, archive old sessions). Implement data export to separate files by date range.

## Dependencies at Risk

**sql.js (sql-wasm.js):**
- Risk: Using vendored copy at `public/sql-wasm.js` (194 lines), not tracked in package.json
- Impact: No automatic updates for security fixes. Version unclear (no version comment in file).
- Migration plan: Add sql.js as explicit dependency in package.json. Or migrate to wa-sqlite for better WASM integration and persistence options. Consider official WASM SQLite builds.

**dioxus = "0.7.0":**
- Risk: Using exact version pin, not SemVer range
- Impact: Won't receive patch updates automatically
- Migration plan: Change to "0.7" to receive patch updates, or actively monitor for 0.7.x releases

**wasm-bindgen = "=0.2.106":**
- Risk: Exact version pin with `=` prefix
- Impact: Manually pinned, suggesting compatibility issues with newer versions
- Migration plan: Test with newer wasm-bindgen versions when upgrading dependencies. May require coordination with Dioxus version.

## Missing Critical Features

**No Data Backup/Export:**
- Problem: Users cannot export their workout data to a portable format
- Blocks: Data migration between devices, data analysis in external tools, long-term archival
- Priority: High - data loss risk if file is corrupted or lost

**No Session Recovery:**
- Problem: If browser crashes during active session, in-progress sets are lost
- Blocks: Users losing data during workouts
- Priority: Medium - auto-save after each set mitigates but doesn't eliminate risk

**No Undo Functionality:**
- Problem: Logged set cannot be deleted or edited
- Blocks: Users cannot correct mistakes in logged data
- Priority: Medium - impacts data quality if user makes input error

**No Historical Data View:**
- Problem: Cannot view past sessions or track progress over time
- Blocks: Core value proposition of workout tracking (seeing improvement)
- Priority: High - app is incomplete without viewing history

## Test Coverage Gaps

**WASM JS Interop Layer:**
- What's not tested: All of `src/state/db.rs` and `src/state/file_system.rs` JS boundary calls
- Files: `src/state/db.rs`, `src/state/file_system.rs`
- Risk: JS errors, permission issues, quota exceeded, file corruption all untested
- Priority: High - these are primary failure modes in production

**Workout State Machine:**
- What's not tested: State transitions, concurrent operations, error recovery
- Files: `src/state/workout_state.rs`
- Risk: State corruption, race conditions, initialization failures could go undetected
- Priority: High - core application logic with complex state management

**UI Input Validation:**
- What's not tested: Form validation, input parsing, error display
- Files: `src/app.rs` lines 160-527
- Risk: Invalid user inputs causing silent failures or data corruption
- Priority: Medium - currently relies on manual testing

**Browser API Fallbacks:**
- What's not tested: LocalStorage fallback when File System Access API unavailable
- Files: `src/state/file_system.rs` lines 213-217, 283-340
- Risk: Fallback path may be broken without detection
- Priority: Medium - affects compatibility with older browsers

---

*Concerns audit: 2026-02-25*
