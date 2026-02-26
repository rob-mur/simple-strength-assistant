# Phase 2: Debug and Fix File Picker - Research

**Researched:** 2026-02-26
**Domain:** File System Access API, WASM-JS Interop, Dioxus Web Platform
**Confidence:** HIGH

## Summary

This phase focuses on debugging and fixing the file picker functionality in a Dioxus 0.7 WASM application that uses the File System Access API via wasm-bindgen. The codebase manually implements `showSaveFilePicker()` calls through `js_sys::Reflect` because web-sys doesn't expose these APIs by default - they require the `web_sys_unstable_apis` configuration flag.

The implementation pattern uses: Rust calls JS functions via js_sys::Reflect → JS API (showSaveFilePicker) → Promise handling via wasm-bindgen-futures → FileSystemFileHandle → IndexedDB storage for persistence.

**Primary recommendation:** Focus debugging on three critical areas: (1) User gesture/transient activation requirements - `showSaveFilePicker` MUST be called from a user interaction event or it throws SecurityError, (2) Permission prompt behavior - verify queryPermission/requestPermission workflow when retrieving cached handles from IndexedDB, (3) Browser console for detailed error messages now that Dioxus logger is initialized.

## Phase Requirements

<phase_requirements>
| ID | Description | Research Support |
|----|-------------|-----------------|
| DB-01 | File picker dialog appears when user needs to select database | User gesture requirements, transient activation timing, SecurityError debugging, browser DevTools console visibility |
| DB-02 | User can successfully select a .sqlite or .db file from their filesystem | File type filtering in showSaveFilePicker options, accept/types configuration, browser compatibility (Chrome/Edge/Safari supported, Firefox not supported) |
| DB-04 | User can grant File System Access API permissions when prompted | Permission state machine (prompt/granted/denied), queryPermission() for checking state, requestPermission() for prompting, persistent permissions in Chrome 122+ |
| ERR-01 | File picker errors are logged to console with clear messages | Error handling patterns for NotAllowedError (permission denied), SecurityError (no user gesture), console.error() for WASM debugging, Dioxus logger integration |
| ERR-02 | Permission denied shows user-friendly error message | NotAllowedError detection, FileSystemError::PermissionDenied mapping, UI state transitions to InitializationState::Error |
| ERR-04 | WASM-JS boundary errors include stack traces | JsValue error formatting, wasm-bindgen error conversion, browser DevTools source maps for WASM debugging |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wasm-bindgen | 0.2.106 | Rust-JS interop | Only mature solution for Rust WASM ↔ JS communication, handles Promise conversions, type marshalling |
| wasm-bindgen-futures | 0.4 | Promise handling | Essential for async JS APIs (showSaveFilePicker returns Promise), integrates with Rust async/await |
| js-sys | 0.3 | JS standard library | Provides Reflect API, Function, Object, Array types for dynamic JS calls when web-sys lacks bindings |
| web-sys | 0.3 | Web APIs | Type-safe bindings to browser APIs, requires explicit features and web_sys_unstable_apis flag for File System Access |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| console_error_panic_hook | Latest | Panic logging | Development only - routes Rust panics to browser console instead of cryptic WASM errors |
| tracing-wasm | Via dioxus::logger | Log routing | Already used - bridges log::* calls to console.log/warn/error with proper formatting |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| js_sys::Reflect | web-sys with unstable flag | web-sys requires RUSTFLAGS=--cfg=web_sys_unstable_apis, adds build complexity but provides type safety |
| Manual IndexedDB calls | gloo-storage only | gloo-storage can't persist FileSystemFileHandle (not serializable to JSON), need IndexedDB structured clone |
| File System Access API | Download links (fallback) | Already implemented - use_fallback mode with LocalStorage, but loses persistent file access |

**Installation:**
Already present in Cargo.toml. No additional dependencies needed.

## Architecture Patterns

### Recommended Project Structure
Current structure is appropriate:
```
src/
├── state/
│   ├── file_system.rs    # FileSystemManager, prompt_for_file, handle persistence
│   ├── db.rs              # Database wrapper, sql.js integration
│   └── workout_state.rs   # Orchestrates initialization flow
├── app.rs                 # UI states (SelectingFile, Error display)
└── main.rs                # Logger initialization (already done in Phase 1)
```

### Pattern 1: User Gesture Requirement for showSaveFilePicker

**What:** File System Access API requires "transient user activation" - the picker MUST be called within ~5 seconds of a user interaction (click, touch, keyboard).

**When to use:** Every call to showSaveFilePicker() must originate from an event handler.

**Example:**
```rust
// CORRECT: Called from onclick handler
rsx! {
    button {
        onclick: move |_| {
            spawn(async move {
                // Within ~4900ms of click = has transient activation
                file_manager.prompt_for_file().await
            });
        },
        "Select Database"
    }
}

// WRONG: Called automatically on mount
use_effect(move || {
    spawn(async move {
        // No user gesture = SecurityError: Must be handling a user gesture
        file_manager.prompt_for_file().await
    });
});
```

**Source:** [File System Access Spec - Transient Activation](https://wicg.github.io/file-system-access/) states "If the global object does not have transient activation, then it throws a 'SecurityError' DOMException."

### Pattern 2: Permission State Machine for Cached Handles

**What:** FileSystemFileHandle persisted in IndexedDB likely resolves with "prompt" permission state on retrieval, requiring explicit permission request.

**When to use:** After retrieving a handle from IndexedDB during app initialization.

**Example:**
```rust
pub async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError> {
    let handle = retrieve_file_handle().await;

    if !handle.is_null() && !handle.is_undefined() {
        // Handle exists in IndexedDB, but permission may have expired

        // JavaScript equivalent: handle.queryPermission({ mode: 'readwrite' })
        let query_permission = js_sys::Reflect::get(&handle, &JsValue::from_str("queryPermission"))?
            .dyn_into::<js_sys::Function>()?;

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &JsValue::from_str("mode"), &JsValue::from_str("readwrite"))?;

        let permission_promise = query_permission.call1(&handle, &options)?;
        let permission_state = JsFuture::from(js_sys::Promise::from(permission_promise)).await?;

        let state_str = permission_state.as_string().unwrap_or_default();

        match state_str.as_str() {
            "granted" => {
                // Good to use immediately
                self.handle = Some(handle);
                Ok(true)
            }
            "prompt" => {
                // Need to request permission (requires user gesture!)
                // Chrome 122+: May auto-grant if persistent permission previously granted
                let request_permission = js_sys::Reflect::get(&handle, &JsValue::from_str("requestPermission"))?
                    .dyn_into::<js_sys::Function>()?;

                let result = request_permission.call1(&handle, &options)?;
                let new_state = JsFuture::from(js_sys::Promise::from(result)).await?;

                if new_state.as_string().unwrap_or_default() == "granted" {
                    self.handle = Some(handle);
                    Ok(true)
                } else {
                    Ok(false) // User denied
                }
            }
            "denied" => {
                // User explicitly denied, clear stale handle
                clear_file_handle().await;
                Ok(false)
            }
            _ => Ok(false)
        }
    } else {
        Ok(false)
    }
}
```

**Source:** [MDN FileSystemHandle.queryPermission()](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemHandle/queryPermission) and [Chrome Persistent Permissions Blog](https://developer.chrome.com/blog/persistent-permissions-for-the-file-system-access-api)

### Pattern 3: Debugging WASM-JS Boundary Errors

**What:** Browser DevTools console is the primary debugging tool for WASM apps. All JS errors, Rust log::* calls, and JsValue errors appear here.

**When to use:** Throughout debugging - console is your source of truth.

**Example:**
```rust
// Good error handling pattern (already in codebase)
let promise = picker_fn.call1(&window, &options).map_err(|e| {
    let error_string = format!("{:?}", e);
    web_sys::console::error_1(&format!("[FileSystem] Error: {}", error_string).into());

    // Parse error type from string representation
    if error_string.to_lowercase().contains("permission")
        || error_string.to_lowercase().contains("notallowederror")
    {
        FileSystemError::PermissionDenied
    } else if error_string.to_lowercase().contains("securityerror")
        || error_string.to_lowercase().contains("user gesture")
    {
        FileSystemError::SecurityError // Add this variant
    } else {
        FileSystemError::UserCancelled
    }
})?;
```

**Browser DevTools tips:**
- F12 → Console tab shows all logs
- Filter by "FileSystem" or "DB" prefixes to track initialization
- Check Network tab → Filter by "wasm" to verify WASM module loaded
- Sources tab → Can set breakpoints in JS code (file-handle-storage.js, db-module.js)

**Source:** [Debugging WebAssembly with Modern Tools](https://developer.chrome.com/blog/wasm-debugging-2020)

### Anti-Patterns to Avoid

- **Processing data before showSaveFilePicker()**: Getting data ready first, then calling picker loses user gesture. Call picker FIRST, then process data.
- **Assuming cached handle works**: Always check permission state with queryPermission() - don't assume "handle exists = can use".
- **Silent errors**: Every await point needs .map_err() with console logging - WASM errors are opaque without explicit logging.
- **Missing web_sys_unstable_apis flag**: If switching from js_sys::Reflect to web-sys bindings, MUST add RUSTFLAGS or web-sys types won't exist at compile time.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FileSystemFileHandle persistence | Manual serialization | IndexedDB structured clone (already implemented) | FileSystemFileHandle uses structured clone algorithm - IndexedDB handles it natively |
| Permission prompts | Custom permission UI | Browser native queryPermission/requestPermission | Browser handles permission state, remembers user choices, can't be bypassed |
| Fallback file storage | Custom file format | sql.js export + LocalStorage (already implemented) | sql.js provides standard SQLite format, LocalStorage is universal fallback |
| Error message localization | Manual translations | Browser error messages | File System Access API errors come pre-localized from browser |

**Key insight:** File System Access API is a security-sensitive API - browser handles permission UI, state persistence, and user choice memory. Don't try to work around it, work with it.

## Common Pitfalls

### Pitfall 1: SecurityError - No User Gesture

**What goes wrong:** `showSaveFilePicker()` throws `SecurityError: Must be handling a user gesture to show a file picker.`

**Why it happens:**
- Called from use_effect/onMount without user interaction
- Called after async operation that took > 4900ms (transient activation expired)
- Called from setTimeout/interval (loses activation)

**How to avoid:**
- Only call from onclick/ontouch/onkeydown handlers
- If async processing needed, call picker FIRST, get handle, THEN process
- For initialization flow: Show UI with "Select Database" button instead of auto-prompting

**Warning signs:**
- Console error contains "SecurityError" and "user gesture"
- File picker never appears, no permission prompt
- Works when manually triggering but not on page load

**Likely cause in current codebase:** `WorkoutStateManager::setup_database()` is called from `use_effect` in app.rs (line 11-17), which runs on mount without user gesture. When `!has_cached`, it calls `prompt_for_file()` immediately → SecurityError.

### Pitfall 2: NotAllowedError - Permission Denied After Cache

**What goes wrong:** Cached handle exists in IndexedDB, but operations fail with `NotAllowedError: User denied permission` even though user previously granted access.

**Why it happens:**
- Permission state degraded to "prompt" (browser restarted, time elapsed, site data cleared)
- queryPermission() not called before using handle
- requestPermission() requires user gesture but called without one

**How to avoid:**
- Always call queryPermission() after retrieving from IndexedDB
- If state is "prompt", need user gesture to call requestPermission()
- Show UI: "Database access requires permission" with button to re-prompt

**Warning signs:**
- File picker doesn't show, but handle exists
- Console error: "NotAllowedError"
- read_file() or write_file() fails immediately

**Detection in code:**
```rust
// Current code checks handle exists but not permission state
if !handle.is_null() && !handle.is_undefined() {
    self.handle = Some(handle);
    Ok(true) // Assumes it's usable!
}

// Should add: queryPermission() → if "prompt" → requestPermission() flow
```

### Pitfall 3: Firefox Compatibility - showSaveFilePicker Not Supported

**What goes wrong:** Application works in Chrome/Edge/Safari but fails completely in Firefox with `showSaveFilePicker is not defined`.

**Why it happens:** Firefox deliberately doesn't support File System Access API due to security philosophy differences. The is_file_system_api_supported() check should catch this, but error handling might not be user-friendly.

**How to avoid:**
- Already have fallback: `use_fallback` mode with LocalStorage
- Ensure is_file_system_api_supported() properly detects Firefox
- Show clear message: "Firefox doesn't support persistent file access, using browser storage"

**Warning signs:**
- TypeError: window.showSaveFilePicker is not a function
- User reports work in Chrome but not Firefox
- is_file_system_api_supported() returns false

**Browser support:**
- ✅ Chrome 86+
- ✅ Edge 86+
- ✅ Safari 15.4+
- ❌ Firefox (all versions as of 2026)

**Source:** [Can I Use - showSaveFilePicker](https://caniuse.com/mdn-api_window_showsavefilepicker)

### Pitfall 4: web_sys_unstable_apis Flag Not Set

**What goes wrong:** If trying to use web-sys types directly (FileSystemFileHandle, FileSystemWritableFileStream), compilation fails with "type not found".

**Why it happens:** File System Access API is not in web-sys stable API surface. Requires `RUSTFLAGS=--cfg=web_sys_unstable_apis`.

**How to avoid:**
- Current approach (js_sys::Reflect) doesn't need flag - works with JsValue
- If migrating to typed web-sys: Create .cargo/config.toml with [build] rustflags = ["--cfg=web_sys_unstable_apis"]
- Or stick with current js_sys::Reflect approach (more verbose but no build config)

**Warning signs:**
- cannot find type `FileSystemFileHandle` in crate `web_sys`
- web-sys feature enabled in Cargo.toml but type doesn't exist
- Works for other developers but not on your machine (different rustflags)

**Current status:** Project uses js_sys::Reflect (line 120-173 in file_system.rs), so this pitfall doesn't apply unless implementation changes.

## Code Examples

Verified patterns from official sources and current codebase analysis:

### Example 1: Proper User Gesture Flow for File Picker

```rust
// In app.rs - InitializationState::SelectingFile UI
InitializationState::SelectingFile => {
    rsx! {
        div {
            class: "card bg-base-100 shadow-xl max-w-md",
            div {
                class: "card-body",
                h2 { "Select Database File" }
                p { "Please select or create a database file to store your workout data." }

                // KEY: Button provides user gesture
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let state_for_select = workout_state.clone();
                        spawn(async move {
                            // Within user gesture context - showSaveFilePicker works
                            let mut fs = FileSystemManager::new();
                            match fs.prompt_for_file().await {
                                Ok(_) => {
                                    // Continue initialization with valid handle
                                    state_for_select.set_initialization_state(InitializationState::Initializing);
                                    // ... rest of setup
                                }
                                Err(e) => {
                                    WorkoutStateManager::handle_error(&state_for_select, e.to_string());
                                }
                            }
                        });
                    },
                    "Select Database Location"
                }
            }
        }
    }
}
```

**Source:** Current codebase pattern (app.rs lines 53-75), modified to include explicit button interaction.

### Example 2: Complete Permission Check for Cached Handles

```javascript
// In file-handle-storage.js - Enhanced retrieveFileHandle with permission checking
export async function retrieveFileHandle() {
    try {
        const db = await openDB();
        const transaction = db.transaction(STORE_NAME, 'readonly');
        const store = transaction.objectStore(STORE_NAME);

        const handle = await new Promise((resolve, reject) => {
            const request = store.get(HANDLE_KEY);
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });

        db.close();

        if (!handle) {
            console.log('[FileHandleStorage] No handle in IndexedDB');
            return null;
        }

        // CRITICAL: Check permission state
        const options = { mode: 'readwrite' };
        const permission = await handle.queryPermission(options);
        console.log('[FileHandleStorage] Permission state:', permission);

        if (permission === 'granted') {
            return handle;
        }

        if (permission === 'prompt') {
            // Chrome 122+ may auto-grant if user checked "Remember this choice"
            console.log('[FileHandleStorage] Requesting permission...');
            const requestedPermission = await handle.requestPermission(options);

            if (requestedPermission === 'granted') {
                return handle;
            }

            console.warn('[FileHandleStorage] Permission denied by user');
            await clearFileHandle();
            return null;
        }

        // permission === 'denied'
        console.warn('[FileHandleStorage] Permission permanently denied, clearing handle');
        await clearFileHandle();
        return null;

    } catch (error) {
        console.error('[FileHandleStorage] Error retrieving handle:', error);
        // Handle may be invalid (file deleted, drive disconnected, etc.)
        await clearFileHandle();
        return null;
    }
}
```

**Source:** Pattern derived from [Chrome Persistent Permissions Guide](https://developer.chrome.com/blog/persistent-permissions-for-the-file-system-access-api) and [MDN queryPermission documentation](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemHandle/queryPermission)

### Example 3: Comprehensive Error Handling at WASM Boundary

```rust
// In file_system.rs - Enhanced prompt_for_file with detailed error detection
pub async fn prompt_for_file(&mut self) -> Result<FileHandle, FileSystemError> {
    web_sys::console::log_1(&"[FileSystem] Opening file picker dialog...".into());

    let window = window().ok_or(FileSystemError::NotSupported)?;

    // Check if API exists (Firefox detection)
    let show_save_file_picker = js_sys::Reflect::get(&window, &JsValue::from_str("showSaveFilePicker"))
        .map_err(|_| {
            web_sys::console::warn_1(&"[FileSystem] showSaveFilePicker not available, using fallback".into());
            FileSystemError::NotSupported
        })?;

    if show_save_file_picker.is_undefined() {
        web_sys::console::warn_1(&"[FileSystem] File System Access API not supported in this browser".into());
        return Err(FileSystemError::NotSupported);
    }

    let picker_fn = show_save_file_picker
        .dyn_ref::<js_sys::Function>()
        .ok_or(FileSystemError::NotSupported)?;

    // Build options object
    let options = js_sys::Object::new();
    let types_array = js_sys::Array::new();
    let type_obj = js_sys::Object::new();

    js_sys::Reflect::set(&type_obj, &JsValue::from_str("description"), &JsValue::from_str("SQLite Database"))?;

    let accept_obj = js_sys::Object::new();
    let extensions_array = js_sys::Array::new();
    extensions_array.push(&JsValue::from_str(".sqlite"));
    extensions_array.push(&JsValue::from_str(".db"));

    js_sys::Reflect::set(&accept_obj, &JsValue::from_str("application/x-sqlite3"), &extensions_array)?;
    js_sys::Reflect::set(&type_obj, &JsValue::from_str("accept"), &accept_obj)?;
    types_array.push(&type_obj);

    js_sys::Reflect::set(&options, &JsValue::from_str("types"), &types_array)?;
    js_sys::Reflect::set(&options, &JsValue::from_str("suggestedName"), &JsValue::from_str("workout_data.sqlite"))?;

    // Call picker - MUST be within user gesture
    let promise = picker_fn.call1(&window, &options).map_err(|e| {
        let error_string = format!("{:?}", e);
        web_sys::console::error_1(&format!("[FileSystem] File picker call failed: {}", error_string).into());

        // Parse specific error types
        let error_lower = error_string.to_lowercase();

        if error_lower.contains("securityerror") || error_lower.contains("user gesture") {
            web_sys::console::error_1(&"[FileSystem] SecurityError: File picker requires user gesture (button click)".into());
            FileSystemError::SecurityError
        } else if error_lower.contains("notallowederror") || error_lower.contains("permission") {
            web_sys::console::error_1(&"[FileSystem] NotAllowedError: User denied permission".into());
            FileSystemError::PermissionDenied
        } else if error_lower.contains("abort") {
            web_sys::console::log_1(&"[FileSystem] User cancelled file selection".into());
            FileSystemError::UserCancelled
        } else {
            FileSystemError::JsError(error_string)
        }
    })?;

    // Await Promise
    let handle = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| {
            let error_string = format!("{:?}", e);
            web_sys::console::error_1(&format!("[FileSystem] Promise rejected: {}", error_string).into());

            let error_lower = error_string.to_lowercase();
            if error_lower.contains("notallowederror") || error_lower.contains("permission") {
                FileSystemError::PermissionDenied
            } else {
                FileSystemError::UserCancelled
            }
        })?;

    web_sys::console::log_1(&"[FileSystem] File handle obtained successfully".into());

    // Store in IndexedDB
    let store_result = store_file_handle(handle.clone()).await;
    if !store_result.is_truthy() {
        web_sys::console::warn_1(&"[FileSystem] Warning: Failed to persist file handle to IndexedDB".into());
    }

    self.handle = Some(handle);
    Ok(FileHandle { cached: true })
}
```

**Source:** Enhanced version of current codebase (file_system.rs lines 110-211) with explicit error type detection.

### Example 4: Initialization Flow with User Gesture Handling

```rust
// In workout_state.rs - Modified setup_database to respect user gesture requirement
impl WorkoutStateManager {
    pub async fn setup_database(state: &WorkoutState) -> Result<(), String> {
        web_sys::console::log_1(&"[DB Init] Starting database setup...".into());

        {
            let mut inner = state.inner.try_borrow_mut()
                .map_err(|_| "Failed to access state".to_string())?;

            if inner.initialization_state == InitializationState::Initializing {
                return Err("Already initializing".to_string());
            }

            inner.initialization_state = InitializationState::Initializing;
        }

        let mut file_manager = FileSystemManager::new();
        let has_cached = file_manager.check_cached_handle().await
            .map_err(|e| format!("Failed to check cached handle: {}", e))?;

        if !has_cached {
            // CHANGE: Don't prompt immediately - requires user gesture
            // Set state to SelectingFile and let UI handle the button click
            web_sys::console::log_1(&"[DB Init] No cached handle, showing file selection UI".into());
            state.set_initialization_state(InitializationState::SelectingFile);

            // Return here - UI will call prompt_for_file() from button onclick
            // which provides the required user gesture
            return Err("Waiting for user to select file".to_string());
        }

        // Has cached handle with valid permissions, continue initialization
        web_sys::console::log_1(&"[DB Init] Using cached handle".into());

        let file_data = if file_manager.has_handle() {
            match file_manager.read_file().await {
                Ok(data) if !data.is_empty() => Some(data),
                _ => None
            }
        } else {
            None
        };

        let mut database = Database::new();
        database.init(file_data).await
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        {
            let mut inner = state.inner.try_borrow_mut()
                .map_err(|e| format!("Failed to borrow state: {}", e))?;
            inner.database = Some(database);
            inner.file_manager = Some(file_manager);
        }

        state.set_initialization_state(InitializationState::Ready);
        web_sys::console::log_1(&"[DB Init] Setup complete".into());

        Ok(())
    }
}
```

**Source:** Modified version of current workout_state.rs (lines 120-218) to handle user gesture requirement.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Download links with `<a download>` | File System Access API with persistent handles | Chrome 86 (Oct 2020) | Users can edit files in place without re-downloading/uploading |
| Permission prompts every session | Persistent permissions (Chrome 122+) | Chrome 122 (Jan 2024) | Users can grant "remember this choice" to avoid repeated prompts |
| web-sys requires experimental flag | Still requires `web_sys_unstable_apis` | Ongoing (2026) | File System Access API not in stable web-sys, use js_sys::Reflect as workaround |
| Manual permission checking | Auto-retry with requestPermission() | Current best practice | Browsers can auto-grant if user previously chose persistent permission |

**Deprecated/outdated:**
- **FileSystem API (deprecated)**: Old Chrome-specific API with `window.requestFileSystem()` - replaced by File System Access API in 2020
- **File and Directory Entries API**: Drag-and-drop focused API - File System Access supersedes it
- **Assuming web-sys types available**: They're not without flag, current js_sys::Reflect approach is correct

## Open Questions

1. **Current error in console**
   - What we know: Phase 1 enabled console logging with Dioxus logger, console should now show initialization errors
   - What's unclear: What specific error appears when file picker fails? SecurityError? NotAllowedError? Other?
   - Recommendation: First debugging step is to check browser console with DevTools open, look for [FileSystem] or [DB Init] prefixed errors

2. **Where prompt_for_file is called from**
   - What we know: workout_state.rs setup_database() calls prompt_for_file() when !has_cached (line 161)
   - What's unclear: Is setup_database() called from use_effect (no user gesture) or from button click (has user gesture)?
   - Recommendation: Check app.rs line 11-17 - if use_effect calls setup_database directly → SecurityError. Need UI button flow instead.

3. **Browser being tested**
   - What we know: Firefox doesn't support File System Access API, code has is_file_system_api_supported() check
   - What's unclear: Is testing happening in Firefox (would fail immediately) or Chrome/Edge/Safari (should work with user gesture)?
   - Recommendation: Verify browser with DevTools → Console → `typeof window.showSaveFilePicker` - should be "function" in Chrome/Edge/Safari, "undefined" in Firefox

4. **Permission state of cached handles**
   - What we know: check_cached_handle() retrieves from IndexedDB but doesn't verify permission state
   - What's unclear: Are cached handles failing due to expired permissions (common after browser restart)?
   - Recommendation: Add queryPermission() call to check_cached_handle(), log permission state to console

## Sources

### Primary (HIGH confidence)
- [File System Access API Specification (WICG)](https://wicg.github.io/file-system-access/) - Official spec defining user gesture requirements, permission model
- [MDN Window.showSaveFilePicker()](https://developer.mozilla.org/en-US/docs/Web/API/Window/showSaveFilePicker) - API documentation and browser compatibility
- [MDN FileSystemHandle.queryPermission()](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemHandle/queryPermission) - Permission checking API
- [Chrome File System Access Guide](https://developer.chrome.com/docs/capabilities/web-apis/file-system-access) - Best practices and examples
- [Chrome Persistent Permissions Blog](https://developer.chrome.com/blog/persistent-permissions-for-the-file-system-access-api) - Chrome 122+ persistent permission feature
- [wasm-bindgen Guide - Console Logging](https://wasm-bindgen.github.io/wasm-bindgen/examples/console-log.html) - WASM debugging patterns
- [Debugging WebAssembly 2020](https://developer.chrome.com/blog/wasm-debugging-2020) - Browser DevTools for WASM

### Secondary (MEDIUM confidence)
- [Can I Use - showSaveFilePicker](https://caniuse.com/mdn-api_window_showsavefilepicker) - Browser compatibility data (verified with MDN)
- [Medium: User Activation and Browser Protections](https://medium.com/@julianlannoo/unraveling-user-activation-and-browser-protections-5c229f61ec37) - Transient activation explanation
- [Transloadit: Persistent File Handling](https://transloadit.com/devtips/persistent-file-handling-with-the-file-system-access-api/) - Practical implementation guide
- [xjavascript.com: File System Access API Guide](https://www.xjavascript.com/blog/file-system-access-api-typescript/) - TypeScript examples applicable to JS integration

### Tertiary (LOW confidence - anecdotal)
- [GitHub Issue: showSaveFilePicker not supported in Firefox](https://github.com/DavidNHill/JSMinesweeper/issues/11) - Community confirmation of Firefox lack of support
- [GitHub Discussion: wasm-bindgen FileSystemDirectoryHandle](https://github.com/wasm-bindgen/wasm-bindgen/discussions/4054) - Example of js_sys::Reflect pattern

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - wasm-bindgen/js-sys are established, File System Access API is stable in Chrome/Edge/Safari
- Architecture: HIGH - Current implementation pattern (js_sys::Reflect) is correct approach, verified by working code structure
- Pitfalls: HIGH - All four pitfalls are documented issues with File System Access API, verified by official sources and error message patterns

**Research date:** 2026-02-26
**Valid until:** 60 days (File System Access API is stable, but browser implementations evolving - Chrome 122 added persistent permissions)
