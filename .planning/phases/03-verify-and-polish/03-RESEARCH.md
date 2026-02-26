# Phase 3: Verify and Polish - Research

**Researched:** 2026-02-26
**Domain:** End-to-End Testing, State Persistence, Error UX, Browser Storage APIs
**Confidence:** HIGH

## Summary

Phase 3 focuses on verifying the complete database initialization flow works reliably end-to-end and polishing error handling for common failure modes. Phase 2 successfully fixed the file picker issue - users can now select database files via button click with proper user gesture handling, cached handles are verified with queryPermission/requestPermission, and the UI reactively transitions through initialization states.

What remains is comprehensive end-to-end verification to ensure: (1) File handle persistence across browser refresh works correctly (DB-03), (2) Database initialization completes successfully after file selection in all scenarios (DB-05), (3) LocalStorage fallback activates properly when File System Access API is unavailable (DB-06), and (4) User-friendly error messages surface for common failure modes like file format validation errors (ERR-03).

**Primary recommendation:** Focus on manual browser testing for the four pending requirements, as they involve browser-level state persistence (IndexedDB), cross-session scenarios (refresh testing), and platform-specific behavior (API availability). Add file format validation error handling with user-friendly messages, and verify fallback storage works in Firefox/unsupported browsers.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DB-03 | Selected file handle persists across browser sessions | IndexedDB structured clone for FileSystemFileHandle persistence (already implemented in file-handle-storage.js), queryPermission/requestPermission workflow in retrieveFileHandle (Phase 2), manual testing required to verify cross-refresh behavior |
| DB-05 | Database initialization completes successfully after file selection | sql.js initialization with file data (db-module.js), Database::init with table creation (db.rs), inline initialization after file selection (app.rs lines 106-119), error handling throughout chain, manual testing of success path |
| DB-06 | LocalStorage fallback works when API unavailable | is_file_system_api_supported() check (file_system.rs line 87), use_fallback_storage() implementation (line 253), read_from_fallback/write_to_fallback (lines 323-379), LocalStorage via gloo-storage, Firefox/Safari private mode testing required |
| ERR-03 | File format validation errors are surfaced to user | SQLite magic number validation in read_file() returns FileSystemError::InvalidFormat (file_system.rs line 317), needs user-friendly error message mapping in UI error state (app.rs lines 149-202), currently returns technical error string |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wasm-bindgen-test | 0.3 | WASM test runner | Only mature solution for running Rust WASM tests in browser environment, integrates with cargo test |
| gloo-storage | 0.3 | LocalStorage wrapper | Already in use, provides type-safe LocalStorage/SessionStorage access with Serde integration |
| IndexedDB API | Native | Persistent storage | Browser native structured storage, required for FileSystemFileHandle persistence (can't be JSON serialized) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| web-sys::console | 0.3 | Browser console logging | Already used extensively - log testing checkpoints for manual verification |
| LocalStorage | Native (via gloo) | Fallback storage | Already implemented - verify it activates in unsupported browsers (Firefox, Safari private mode) |

### Testing Strategy
| Test Type | Implementation | Coverage |
|-----------|----------------|----------|
| Unit tests | wasm-bindgen-test (already exists) | Database operations (db_tests.rs), FileSystem fallback (file_system_tests.rs) - 35 existing tests |
| Integration tests | Manual browser testing | End-to-end initialization flow, permission persistence, cross-session scenarios |
| Browser compatibility | Manual testing matrix | Chrome (File System API), Firefox (LocalStorage fallback), Safari (File System API), Edge (File System API) |

**Note:** Existing test files demonstrate wasm-bindgen-test patterns. Phase 3 testing is primarily **manual browser testing** because:
1. Browser refresh scenarios can't be automated with wasm-bindgen-test (loses test context)
2. IndexedDB/LocalStorage state persistence requires cross-session verification
3. Permission dialog interactions require manual user action
4. Browser compatibility testing requires multiple browser environments

## Architecture Patterns

### Current Test Structure
```
src/state/
├── db_tests.rs              # 14 wasm-bindgen tests for Database (lines 1-278)
├── file_system_tests.rs     # 12 wasm-bindgen tests for FileSystemManager (lines 1-227)
└── workout_state.rs         # 3 inline unit tests for predictions (lines 366-434)
```

### Pattern 1: Manual Browser Testing Protocol

**What:** Systematic manual verification of end-to-end flows that involve browser-level state, permissions, and cross-session behavior.

**When to use:** Testing features that involve:
- Browser refresh/reload (lose WASM instance state)
- IndexedDB persistence (cross-session)
- Permission dialogs (require user interaction)
- Browser-specific APIs (File System Access vs fallback)

**Example Test Checklist:**
```markdown
## Test 1: First-time File Selection
1. Open app in fresh browser profile (no cached data)
2. Verify: Shows "Select Database Location" button
3. Click button
4. Verify: Native file picker appears
5. Select/create "test.sqlite"
6. Verify: Console logs "[UI] Database initialized successfully"
7. Verify: UI transitions to Ready state (shows "Start New Workout Session")

## Test 2: Browser Refresh with Cached Handle
1. After Test 1 completion, press F5 (refresh)
2. Verify: Console logs "[FileHandleStorage] Permission state: granted"
3. Verify: No button shown, auto-initializes to Ready state
4. Verify: Can start workout session without re-selecting file

## Test 3: Permission Expired (Restart Browser)
1. After Test 1, close browser completely
2. Reopen browser, navigate to app
3. Verify: Console logs "[FileHandleStorage] Permission state: prompt"
4. Verify: Either (a) auto-grants if persistent permission, or (b) shows button
5. If button: Click to re-grant permission
6. Verify: Returns to Ready state with same file

## Test 4: Invalid File Format
1. Start fresh session
2. Click "Select Database Location"
3. Select non-SQLite file (e.g., .txt, .json)
4. Verify: Console shows "File is not a valid SQLite database"
5. Verify: UI shows error card with user-friendly message
6. Verify: Can click "Retry" to try again

## Test 5: LocalStorage Fallback (Firefox)
1. Open app in Firefox (no File System Access API)
2. Verify: Console logs "[FileSystem] Using fallback storage"
3. Verify: No file picker, auto-initializes to Ready state
4. Start workout, log set
5. Refresh browser
6. Verify: Data persists across refresh (LocalStorage worked)
```

**Source:** Testing pattern adapted from Phase 2 UAT results (02-UAT.md) which successfully verified 5 manual tests.

### Pattern 2: Error Message Enhancement

**What:** Map technical FileSystemError variants to user-friendly messages with actionable guidance.

**Current state:** Errors bubble up as technical strings (e.g., "File is not a valid SQLite database").

**Enhancement needed:** Context-aware error messages with recovery instructions.

**Example Enhancement:**
```rust
// In app.rs - Error state UI enhancement
InitializationState::Error => {
    let error_msg = workout_state.error_message().unwrap_or_else(|| "Unknown error".to_string());

    // Parse error type and provide friendly message + recovery action
    let (friendly_msg, recovery_tip) = match error_msg.as_str() {
        msg if msg.contains("not a valid SQLite database") => (
            "The selected file is not a valid SQLite database.".to_string(),
            "Please select a .sqlite or .db file, or create a new file.".to_string()
        ),
        msg if msg.contains("Permission denied") => (
            "File access permission was denied.".to_string(),
            "Grant file access permission to continue, or use browser storage instead.".to_string()
        ),
        msg if msg.contains("User cancelled") => (
            "File selection was cancelled.".to_string(),
            "Click 'Retry' to select a database location.".to_string()
        ),
        msg if msg.contains("File is too large") => (
            "The selected file is too large (max 100 MB).".to_string(),
            "Please select a smaller database file or contact support.".to_string()
        ),
        _ => (error_msg.clone(), "Click 'Retry' to try again.".to_string())
    };

    rsx! {
        div { class: "alert alert-error",
            h3 { friendly_msg }
            p { class: "text-sm mt-2", recovery_tip }
        }
        button {
            onclick: move |_| { /* retry logic */ },
            "Retry"
        }
    }
}
```

**Source:** UX best practices for error messaging - provide context, explain impact, offer recovery path.

### Pattern 3: Fallback Detection and Messaging

**What:** Clearly communicate to users when fallback storage is active (especially in Firefox).

**Current state:** Fallback activates silently, users don't know their data is in LocalStorage vs file.

**Enhancement needed:** Inform users about storage location and limitations.

**Example Enhancement:**
```rust
// After successful initialization in Ready state
if let Some(file_manager) = workout_state.file_manager() {
    if file_manager.is_using_fallback() {  // Add this method to FileSystemManager
        rsx! {
            div { class: "alert alert-info mb-4",
                svg { /* info icon */ }
                div {
                    h4 { "Using Browser Storage" }
                    p {
                        "Your browser doesn't support persistent file access. ",
                        "Data is stored in browser LocalStorage and won't sync across devices."
                    }
                }
            }
        }
    }
}
```

**Why important:** Users need to understand data portability limitations when fallback is active.

**Source:** Progressive enhancement principle - degrade gracefully with clear communication.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Browser test automation | Custom Selenium/Playwright setup | Manual testing checklist | Cross-session scenarios, permission dialogs, and IndexedDB persistence aren't reliably automatable for WASM apps; manual testing is more effective |
| Error message templating | Complex error message engine | Pattern matching on error strings | Only 4-5 error types to handle, simple match expression is clearer than abstraction |
| Storage backend abstraction | Generic storage trait with multiple backends | Current direct implementation | Only two storage modes (File System API vs LocalStorage), abstraction adds complexity without benefit |
| Database format validation | Custom file format parser | SQLite magic number check (16 bytes) | Already implemented, catches 99% of invalid files instantly, sql.js handles deeper validation |

**Key insight:** Phase 3 is about verification and polish, not building new abstractions. Focus on testing thoroughness and UX refinement rather than architectural changes.

## Common Pitfalls

### Pitfall 1: Assuming IndexedDB Persistence Works Without Testing

**What goes wrong:** File handle appears to persist, but browser clears IndexedDB in certain scenarios (private mode, quota exceeded, user clearing site data).

**Why it happens:**
- IndexedDB has storage quotas (varies by browser)
- Private/incognito mode may not persist across browser restarts
- User can clear site data via browser settings
- Browser may evict data if storage pressure is high

**How to avoid:**
- Test in private/incognito mode explicitly
- Handle retrieveFileHandle() returning null gracefully
- Always check permission state even when handle exists
- Show clear UI when handle is lost: "Database connection lost, please re-select file"

**Warning signs:**
- Works in normal mode, fails in private mode
- Works initially, fails after browser restart
- Console shows "No handle in IndexedDB" on second session
- Permission check returns null despite previous selection

**Detection in code:**
```rust
// In workout_state.rs setup_database
let has_cached = file_manager.check_cached_handle().await
    .map_err(|e| format!("Failed to check cached handle: {}", e))?;

if !has_cached {
    // Handle is gone - could be:
    // 1. First time (expected)
    // 2. IndexedDB cleared (need to inform user)
    // 3. File deleted/drive disconnected (need error message)

    // Don't assume reason - just transition to SelectingFile
    state.set_initialization_state(InitializationState::SelectingFile);
    return Ok(()); // Let user re-select
}
```

### Pitfall 2: Testing Only Happy Path

**What goes wrong:** App works perfectly when everything succeeds, but crashes or shows cryptic errors when user cancels dialogs, selects wrong files, or loses permissions.

**Why it happens:**
- Happy path is easy to test (click button, select file, done)
- Error paths require deliberate effort to trigger
- Developers test with correct files, users test with anything
- Permission errors only occur in cross-session scenarios

**How to avoid:**
- Explicitly test each error scenario in checklist
- Test with intentionally wrong files (.txt, .json, empty file)
- Test cancellation at every dialog
- Test permission denial (click "Block" instead of "Allow")
- Test with file on disconnected drive (if applicable)

**Warning signs:**
- Panic on file selection cancellation
- "Unknown error" displayed for common issues
- Console errors but UI doesn't update
- Retry button doesn't actually retry

**Test coverage checklist:**
```markdown
❌ User clicks "Select Database Location" then cancels → Should stay in SelectingFile state
❌ User selects .txt file → Should show "not a valid SQLite database" error
❌ User selects file then deletes it externally → Should handle gracefully on next session
❌ User denies permission when prompted → Should show permission error with recovery
❌ Browser clears IndexedDB → Should request file selection again
❌ Network drive disconnects with database file → Should show connection lost error
```

### Pitfall 3: Not Testing Fallback Storage in Firefox

**What goes wrong:** App works perfectly in Chrome/Edge, completely fails or behaves differently in Firefox because fallback storage path is untested.

**Why it happens:**
- Developers primarily test in Chrome (most common browser for web dev)
- Firefox doesn't support File System Access API → different code path
- LocalStorage has different characteristics than file handles
- No file picker in Firefox → different initialization flow

**How to avoid:**
- Test in Firefox explicitly (download Firefox if needed)
- Verify console shows "[FileSystem] Using fallback storage"
- Confirm data persists across refresh in Firefox
- Test LocalStorage quota limits (usually 5-10MB)
- Verify no file picker UI appears in Firefox (should auto-initialize)

**Warning signs:**
- TypeError: window.showSaveFilePicker is not a function
- Firefox shows loading spinner forever
- Data doesn't persist in Firefox but works in Chrome
- Different UI flow in Firefox vs Chrome

**Firefox-specific testing:**
```markdown
## Firefox Test Suite
1. Open app in Firefox
2. Verify: No file picker button (fallback auto-activates)
3. Verify: Console logs "[FileSystem] Using fallback storage"
4. Create workout session, log sets
5. Check DevTools → Storage → Local Storage → workout_db_data
6. Verify: Data exists as base64/binary blob
7. Refresh browser
8. Verify: Data persists, session history intact
```

### Pitfall 4: Ignoring File Format Validation Errors

**What goes wrong:** User selects non-SQLite file, gets cryptic sql.js error instead of clear message about file format.

**Why it happens:**
- read_file() validates SQLite magic number → returns InvalidFormat error
- Error bubbles up to UI as technical string
- No user-friendly error message mapping
- User doesn't understand what "SQLite format 3" means

**How to avoid:**
- Check for FileSystemError::InvalidFormat specifically
- Show clear message: "Please select a .sqlite or .db file"
- Offer to create new file instead of selecting existing
- Don't expose technical details (magic numbers, format specs)

**Warning signs:**
- User reports "selected file but got error"
- Error message mentions "SQLite format 3" (too technical)
- No guidance on how to fix the issue
- User repeatedly selects wrong file type

**Current validation (file_system.rs lines 313-318):**
```rust
// Validate SQLite format if file is not empty
if !buffer.is_empty()
    && buffer.len() >= SQLITE_MAGIC_NUMBER.len()
    && !buffer.starts_with(SQLITE_MAGIC_NUMBER)
{
    return Err(FileSystemError::InvalidFormat);
}
```

**Needs user-friendly handling in app.rs:**
```rust
Err(e) if e.to_string().contains("not a valid SQLite database") => {
    WorkoutStateManager::handle_error(&workout_state,
        "Invalid File Format: Please select a .sqlite or .db file, or create a new database.".to_string()
    );
}
```

## Code Examples

### Example 1: Complete Manual Test Protocol

**Phase 3 Test Checklist (Manual Browser Testing):**

```markdown
# Phase 3: End-to-End Verification Test Suite

## Prerequisites
- Chrome/Edge (latest)
- Firefox (latest)
- Clean browser profile (or use incognito)

## Test Suite

### DB-03: Handle Persistence Across Refresh

#### Test 3.1: Normal Refresh with Granted Permission
**Setup:** Fresh browser, no cached data
**Steps:**
1. Navigate to app
2. Click "Select Database Location"
3. Create/select "test-db.sqlite"
4. Grant permission when prompted
5. Wait for "Start New Workout Session" UI
6. Press F5 (refresh page)

**Expected:**
- Console logs: "[FileHandleStorage] Permission state: granted"
- No file selection button shown
- Auto-initializes to Ready state
- "Start New Workout Session" UI visible
- Same file handle reused (no re-selection needed)

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test 3.2: Browser Restart with Permission Prompt
**Setup:** After Test 3.1
**Steps:**
1. Close browser completely (quit application)
2. Reopen browser
3. Navigate to app
4. Observe initialization behavior

**Expected:**
- Console logs: "[FileHandleStorage] Permission state: prompt"
- Either:
  - (a) Auto-grants permission (Chrome 122+ with persistent permission) → Ready state
  - (b) Shows file selection button → User clicks → Permission prompt → Ready state
- After permission granted, app enters Ready state with same file

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test 3.3: Permission Denied Scenario
**Setup:** Fresh browser profile
**Steps:**
1. Navigate to app
2. Click "Select Database Location"
3. Select file, but DENY permission when browser prompts

**Expected:**
- Console logs: "[FileHandleStorage] User denied permission request"
- IndexedDB cleared (no stale handle stored)
- UI shows error: "File access permission was denied"
- Error message includes recovery instruction
- Can click "Retry" to try again

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

### DB-05: Successful Database Initialization

#### Test 5.1: First-Time Initialization (New File)
**Setup:** Fresh browser
**Steps:**
1. Click "Select Database Location"
2. Create NEW file "workout-new.sqlite"
3. Grant permission
4. Wait for initialization

**Expected:**
- Console logs: "[UI] File is empty, creating new database"
- Console logs: "[DB] initDatabase succeeded, creating tables..."
- Console logs: "[DB] Tables created successfully"
- Console logs: "[UI] Database initialized successfully"
- Console logs: "[UI] Setup complete! State is now Ready"
- UI shows "Start New Workout Session" form

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test 5.2: Initialization with Existing File
**Setup:** Use existing valid SQLite file with workout data
**Steps:**
1. Click "Select Database Location"
2. Select EXISTING file "existing-workout.sqlite" (created separately with sql.js)
3. Grant permission
4. Wait for initialization

**Expected:**
- Console logs: "[UI] Read X bytes from file"
- Console logs: "[DB] Database loaded from file data"
- Console logs: "[DB] Tables created successfully" (verifies schema)
- UI enters Ready state
- Can query existing data if any

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

### DB-06: LocalStorage Fallback

#### Test 6.1: Firefox Fallback Activation
**Setup:** Firefox browser (or Chrome with API disabled)
**Steps:**
1. Open app in Firefox
2. Observe console output
3. Observe UI state

**Expected:**
- Console logs: "[FileSystem] Using fallback storage (IndexedDB/LocalStorage)"
- Console logs: "[FileSystem] Fallback storage doesn't need handle caching"
- NO file selection button shown
- Auto-initializes to Ready state immediately
- LocalStorage used for data persistence

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test 6.2: Fallback Data Persistence
**Setup:** After Test 6.1 in Firefox
**Steps:**
1. Start workout session
2. Log 2-3 sets
3. Complete session
4. Refresh browser (F5)
5. Check if data persists

**Expected:**
- After refresh, app initializes to Ready state
- Can start new workout session
- Previous session data persisted (verify in DevTools → Storage → Local Storage)
- key "workout_db_data" exists with binary data

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test 6.3: Private Mode Fallback
**Setup:** Chrome Incognito mode
**Steps:**
1. Open app in Incognito
2. Verify fallback or File System API behavior
3. Log workout data
4. Close incognito window
5. Reopen incognito, navigate to app

**Expected:**
- Data does NOT persist (incognito cleared)
- App starts fresh with no cached handle
- Shows file selection button (File System API mode) OR auto-initializes (fallback mode)

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

### ERR-03: File Format Validation

#### Test ERR-3.1: Invalid File Type (.txt)
**Setup:** Create test.txt with random content
**Steps:**
1. Click "Select Database Location"
2. Select "test.txt" (not a SQLite file)
3. Observe error handling

**Expected:**
- Console logs: "[FileSystem] WASM error" or validation failure
- UI shows error: "Invalid File Format: Please select a .sqlite or .db file"
- Error card visible with friendly message
- "Retry" button available
- App stays in Error state (not crash)

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test ERR-3.2: Empty File
**Setup:** Create empty.sqlite (0 bytes)
**Steps:**
1. Click "Select Database Location"
2. Select empty file
3. Observe initialization

**Expected:**
- Console logs: "[UI] File is empty, creating new database"
- Treated as new database (not error)
- Initializes successfully
- UI enters Ready state

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

#### Test ERR-3.3: Corrupted SQLite File
**Setup:** Create file with SQLite magic number but corrupted data
**Steps:**
1. Create file starting with "SQLite format 3\0" but invalid schema
2. Select this file
3. Observe error handling

**Expected:**
- Passes magic number check (doesn't trigger InvalidFormat)
- sql.js init or table creation fails
- UI shows error: "Database initialization failed"
- Retry button available

**Result:** [ ] Pass [ ] Fail
**Notes:**

---

## Test Summary
- Total tests: 10
- Passed: ___
- Failed: ___
- Blocked: ___

## Issues Found
1.
2.
3.

## Browser Compatibility Matrix
| Browser | Version | File System API | Fallback | Status |
|---------|---------|----------------|----------|--------|
| Chrome  | ___     | [ ] Works      | N/A      | [ ] Pass |
| Edge    | ___     | [ ] Works      | N/A      | [ ] Pass |
| Firefox | ___     | N/A            | [ ] Works| [ ] Pass |
| Safari  | ___     | [ ] Works      | [ ] Works| [ ] Pass |
```

**Source:** Test protocol synthesized from Phase 2 UAT structure (02-UAT.md) and pending Phase 3 requirements.

### Example 2: Enhanced Error Message Handling

**Add to app.rs InitializationState::Error UI:**

```rust
// Current implementation (app.rs lines 149-202) shows raw error string
// Enhanced version provides context and recovery actions

InitializationState::Error => {
    let error_msg = workout_state.error_message()
        .unwrap_or_else(|| "Unknown error occurred".to_string());

    // Parse error type and determine user-friendly message + action
    let error_info = parse_error_for_ui(&error_msg);

    rsx! {
        div {
            class: "flex items-center justify-center h-full",
            div {
                class: "card bg-base-100 shadow-xl max-w-md",
                div {
                    class: "card-body",
                    div {
                        class: "alert alert-error mb-4",
                        svg { /* error icon */ }
                        div {
                            h3 {
                                class: "font-bold",
                                {error_info.title}
                            }
                            p {
                                class: "text-sm mt-2",
                                {error_info.message}
                            }
                            if let Some(tip) = error_info.recovery_tip {
                                p {
                                    class: "text-sm mt-2 font-semibold",
                                    "💡 {tip}"
                                }
                            }
                        }
                    }
                    div {
                        class: "card-actions justify-end",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                spawn(async move {
                                    workout_state.set_error_message(None);
                                    workout_state.set_initialization_state(
                                        InitializationState::NotInitialized
                                    );
                                    if let Err(e) = WorkoutStateManager::setup_database(&workout_state).await {
                                        WorkoutStateManager::handle_error(&workout_state, e);
                                    }
                                });
                            },
                            {error_info.retry_label}
                        }
                    }
                }
            }
        }
    }
}

// Helper struct and function for error parsing
struct ErrorInfo {
    title: String,
    message: String,
    recovery_tip: Option<String>,
    retry_label: String,
}

fn parse_error_for_ui(error_msg: &str) -> ErrorInfo {
    let error_lower = error_msg.to_lowercase();

    if error_lower.contains("not a valid sqlite database") || error_lower.contains("invalid format") {
        ErrorInfo {
            title: "Invalid File Format".to_string(),
            message: "The selected file is not a valid SQLite database.".to_string(),
            recovery_tip: Some("Please select a .sqlite or .db file, or create a new database file.".to_string()),
            retry_label: "Select Different File".to_string(),
        }
    } else if error_lower.contains("permission denied") || error_lower.contains("notallowederror") {
        ErrorInfo {
            title: "Permission Denied".to_string(),
            message: "File access permission was not granted.".to_string(),
            recovery_tip: Some("Grant permission to access the file, or use browser storage instead.".to_string()),
            retry_label: "Grant Permission".to_string(),
        }
    } else if error_lower.contains("user cancelled") {
        ErrorInfo {
            title: "File Selection Cancelled".to_string(),
            message: "No database file was selected.".to_string(),
            recovery_tip: Some("Click below to select where to store your workout data.".to_string()),
            retry_label: "Select File".to_string(),
        }
    } else if error_lower.contains("file is too large") || error_lower.contains("filetoolarge") {
        ErrorInfo {
            title: "File Too Large".to_string(),
            message: "The selected database file exceeds the 100 MB limit.".to_string(),
            recovery_tip: Some("Try selecting a smaller file or export your data to start fresh.".to_string()),
            retry_label: "Select Different File".to_string(),
        }
    } else if error_lower.contains("failed to initialize database") {
        ErrorInfo {
            title: "Database Initialization Failed".to_string(),
            message: "Could not set up the database. The file may be corrupted.".to_string(),
            recovery_tip: Some("Try selecting a different file or creating a new database.".to_string()),
            retry_label: "Try Again".to_string(),
        }
    } else {
        ErrorInfo {
            title: "Initialization Error".to_string(),
            message: error_msg.to_string(),
            recovery_tip: Some("Check your browser console for details and try again.".to_string()),
            retry_label: "Retry".to_string(),
        }
    }
}
```

**Source:** UX best practice - errors should be scannable (bold title), understandable (plain language), and actionable (recovery tip + button).

### Example 3: Storage Mode Indicator for Fallback Users

**Add to app.rs WorkoutInterface (after Ready state is reached):**

```rust
#[component]
fn WorkoutInterface(state: WorkoutState) -> Element {
    let current_session = state.current_session();

    // Check if using fallback storage
    let storage_mode_info = if let Some(fm) = state.file_manager() {
        if fm.is_using_fallback() {  // Need to add this method to FileSystemManager
            Some(rsx! {
                div {
                    class: "alert alert-info mb-4",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        class: "stroke-current shrink-0 w-6 h-6",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        }
                    }
                    div {
                        h4 { class: "font-bold", "Browser Storage Mode" }
                        p { class: "text-sm",
                            "Your data is stored in browser LocalStorage. ",
                            "This works offline but won't sync across devices or browsers."
                        }
                    }
                }
            })
        } else {
            None
        }
    } else {
        None
    };

    rsx! {
        div {
            // Show storage mode info banner if using fallback
            {storage_mode_info}

            // Regular workout interface
            if let Some(session) = current_session {
                ActiveSession { state: state.clone(), session }
            } else {
                StartSessionView { state: state.clone() }
            }
        }
    }
}
```

**Add to file_system.rs:**

```rust
impl FileSystemManager {
    pub fn is_using_fallback(&self) -> bool {
        self.use_fallback
    }
}
```

**Source:** Progressive disclosure - inform users about degraded functionality without blocking core features.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual file upload/download per session | File System Access API with persistent handles | Chrome 86 (Oct 2020) | Users edit in place without re-uploading, Phase 2 implemented this |
| Alert() for all errors | Context-aware error messages with recovery actions | Modern web UX standard (2024+) | Better user experience, reduced support burden |
| Silently fall back to alternative storage | Explicitly communicate storage mode to users | Progressive enhancement best practice | Users understand data portability limitations |
| No cross-browser testing | Test matrix covering Chrome/Firefox/Safari/Edge | CI/CD standard (ongoing) | Catch browser-specific issues before users report them |

**Deprecated/outdated:**
- **No error handling philosophy**: Old apps crashed on unexpected input - modern apps gracefully handle errors with user guidance
- **Developer-centric error messages**: Stack traces and technical jargon shown to users - should show plain language with recovery steps
- **Assuming happy path**: Testing only success cases - should explicitly test error scenarios, cancellations, and edge cases

## Open Questions

1. **Database file size in practice**
   - What we know: 100 MB limit enforced in file_system.rs (line 20), should prevent memory issues
   - What's unclear: Real-world database growth rate - how long until users hit limits?
   - Recommendation: Monitor in Phase 3 testing, adjust if needed (unlikely to matter for v1 scope)

2. **Permission persistence variability**
   - What we know: Chrome 122+ supports persistent permissions, older browsers don't
   - What's unclear: What percentage of users will need to re-grant permission on each browser restart?
   - Recommendation: Test with Chrome 121 and 122+ to observe difference, inform UX decisions

3. **LocalStorage quota in fallback mode**
   - What we know: LocalStorage typically 5-10 MB limit depending on browser
   - What's unclear: How many workout sessions before hitting quota? Need quota exceeded error handling?
   - Recommendation: Test fallback mode with multiple large sessions, add quota check if needed

4. **Safari File System Access API support**
   - What we know: Safari 15.4+ supports API (per research), but implementation may differ
   - What's unclear: Are there Safari-specific quirks or permission UI differences?
   - Recommendation: Test on Safari explicitly during Phase 3, may need Safari-specific error handling

## Validation Architecture

> Note: workflow.nyquist_validation is not set in config.json (defaults to false), but this section documents testing approach for reference.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | wasm-bindgen-test 0.3 + manual browser testing |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `wasm-pack test --headless --chrome` (for unit tests only) |
| Full suite command | Manual test protocol (see Code Examples section) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DB-03 | File handle persists across browser refresh | manual | N/A - requires browser restart | ❌ Manual Test 3.1, 3.2 |
| DB-05 | Database initialization succeeds after file selection | manual + unit | `wasm-pack test --headless --chrome` | ✅ db_tests.rs (test_database_initialization) + Manual Test 5.1, 5.2 |
| DB-06 | LocalStorage fallback works when API unavailable | unit + manual | `wasm-pack test --headless --chrome` | ✅ file_system_tests.rs (test_fallback_storage_write_read) + Manual Test 6.1, 6.2 |
| ERR-03 | File format validation errors surfaced to user | unit + manual | `wasm-pack test --headless --chrome` | ✅ file_system_tests.rs (test_sqlite_format_validation) + Manual Test ERR-3.1, ERR-3.2 |

### Sampling Rate
- **Per task commit:** `cargo build --target wasm32-unknown-unknown` (verify compilation)
- **Per wave merge:** `wasm-pack test --headless --chrome` (run existing unit tests)
- **Phase gate:** Full manual test protocol (10 tests) before marking phase complete

### Wave 0 Gaps
- ✅ Existing test infrastructure adequate (wasm-bindgen-test configured, 27 existing tests)
- ✅ No new test framework needed
- ❌ Manual test protocol document (provided in Code Examples section)
- ❌ User-friendly error message enhancement (code in app.rs)
- ❌ Storage mode indicator component (code in app.rs)

**Test coverage rationale:** Phase 3 requirements involve browser-level behavior (IndexedDB persistence, permission dialogs, cross-session state) that can't be fully automated. The combination of existing unit tests (verify technical functionality) + comprehensive manual test protocol (verify UX and end-to-end flows) provides appropriate coverage.

## Sources

### Primary (HIGH confidence)
- [Phase 2 Research](../.planning/phases/02-debug-and-fix-file-picker/02-RESEARCH.md) - File System Access API details, permission model, user gesture requirements
- [Phase 2 UAT Results](../.planning/phases/02-debug-and-fix-file-picker/02-UAT.md) - Testing protocol that successfully verified Phase 2 implementation
- [wasm-bindgen-test Documentation](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html) - Official test runner for WASM
- Current codebase test files (db_tests.rs, file_system_tests.rs) - Existing test patterns and coverage
- [IndexedDB API Specification](https://w3c.github.io/IndexedDB/) - Storage persistence behavior, structured clone algorithm
- [MDN LocalStorage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage) - Fallback storage API reference
- [File System Access: queryPermission](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemHandle/queryPermission) - Permission state checking (Phase 2 implemented)

### Secondary (MEDIUM confidence)
- [Chrome Storage Quotas](https://web.dev/articles/storage-for-the-web#how_much) - LocalStorage and IndexedDB limits
- [Nielsen Norman Group: Error Message Guidelines](https://www.nngroup.com/articles/error-message-guidelines/) - UX best practices for error messaging
- [Material Design: Error States](https://m2.material.io/design/communication/states.html#error) - Error UI patterns
- [Progressive Enhancement with Storage APIs](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Tutorials/js13kGames/Offline_Service_workers#the_progressive_in_progressive_web_apps) - Fallback storage patterns

### Tertiary (LOW confidence)
- [WebAssembly Testing Strategies](https://developer.mozilla.org/en-US/docs/WebAssembly/Rust_to_Wasm/Testing) - General WASM testing approaches (most recommend manual testing for browser-specific features)
- Community discussions on WASM + browser API testing complexity (general consensus: manual testing required for cross-session scenarios)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - wasm-bindgen-test is the established solution, manual testing is appropriate for browser-specific behavior
- Testing protocol: HIGH - Adapted from successful Phase 2 UAT protocol, covers all pending requirements
- Error handling: HIGH - Standard UX patterns for error messaging, straightforward to implement
- Cross-browser: MEDIUM - Safari testing not yet conducted, may reveal browser-specific quirks

**Research date:** 2026-02-26
**Valid until:** 30 days (testing protocols remain valid, browser behavior stable)
