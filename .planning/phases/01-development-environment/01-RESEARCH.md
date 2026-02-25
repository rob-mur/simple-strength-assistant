# Phase 1: Development Environment - Research

**Researched:** 2026-02-25
**Domain:** Dioxus 0.7 WASM development environment, console debugging, hot-reload
**Confidence:** HIGH

## Summary

Phase 1 requires setting up a Dioxus 0.7 web development environment that enables effective debugging through browser console logs. The project already has most infrastructure in place via devenv.nix (Nix-based development environment) with Rust 1.92.0, dioxus-cli 0.7.2, wasm-bindgen-cli, and binaryen. The existing codebase uses `log` crate macros for logging but lacks explicit logger initialization in main.rs.

Dioxus 0.7 provides first-party logging integration through `dioxus::logger::init()` that automatically configures platform-appropriate loggers (tracing-wasm for web, which outputs to browser console). The development server `dx serve` includes hot-reload for RSX markup/assets and experimental Rust hot-patching via `--hotpatch` flag. Console access requires no special configuration - logs automatically appear in browser DevTools when using Dioxus logger or web-sys console APIs.

**Primary recommendation:** Initialize Dioxus logger in main.rs with Debug level for development, verify dx serve runs successfully, and ensure initialization logs are visible in browser console to meet all four phase requirements (DEV-01 through DEV-04).

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DEV-01 | Development server runs and serves app in browser | dx serve command with existing Dioxus.toml config; devenv.nix already includes dioxus-cli 0.7.2 |
| DEV-02 | Browser console logs are accessible for debugging | Dioxus logger with tracing-wasm outputs to browser console; existing log::* calls will route through logger |
| DEV-03 | WASM compilation succeeds without errors | Rust 1.92.0 with wasm32-unknown-unknown target installed; existing Cargo.toml with wasm-bindgen 0.2.106 |
| DEV-04 | Hot reload works for Rust code changes | dx serve enables RSX hot-reload by default; --hotpatch flag enables experimental Rust hot-patching |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| dioxus | 0.7.0 | Rust web framework compiling to WASM | Official framework for this project; provides React-like components for WASM |
| dioxus-cli (dx) | 0.7.2 | Development server and build tool | Official CLI from DioxusLabs; handles serving, hot-reload, bundling, and WASM compilation |
| wasm-bindgen | 0.2.106 | Rust-JavaScript interop layer | Industry standard for WASM-JS bridging; required by Dioxus web target |
| tracing | (via dioxus::logger) | Structured logging framework | Dioxus first-party logging solution; cross-platform abstractions (tracing-wasm for web) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| web-sys | 0.3 | Browser API bindings | Direct console access via web_sys::console if needed beyond logger |
| console_error_panic_hook | (optional) | Better panic messages in console | Recommended for development to see formatted panic messages vs cryptic RuntimeError |
| binaryen (wasm-opt) | Latest | WASM optimization tool | Automatically used by dx CLI; manual use not needed |
| wasm-bindgen-cli | Latest | Generate JS bindings from WASM | Automatically used by dx CLI; manual invocation not needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| dx serve | trunk + wasm-pack | trunk is older WASM build tool; dx is Dioxus-specific with better hot-reload and integrated tooling |
| dioxus::logger | console_log crate | console_log requires manual setup; Dioxus logger is zero-config and cross-platform |
| dx build | wasm-pack build | wasm-pack is lower-level; dx handles Dioxus-specific requirements like RSX compilation |

**Installation:**
```bash
# Already installed in devenv.nix - no action needed
# For reference, manual installation would be:
cargo install dioxus-cli
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
```

## Architecture Patterns

### Recommended Project Structure
```
simple-strength-assistant/
├── src/
│   ├── main.rs              # Entry point - initialize logger HERE
│   ├── app.rs               # Root component
│   ├── models/              # Domain types
│   └── state/               # State management
├── public/                  # Static assets + JS bridges
├── Dioxus.toml              # Dioxus CLI configuration
├── Cargo.toml               # Rust dependencies
└── index.html               # HTML shell (loads WASM)
```

### Pattern 1: Logger Initialization (Main Entry Point)
**What:** Initialize Dioxus logger before launching app to capture all logs
**When to use:** Always in main.rs before dioxus::launch()
**Example:**
```rust
// Source: https://dioxuslabs.com/learn/0.7/guides/utilities/logging/
use dioxus::prelude::*;
use tracing::Level;

fn main() {
    // Initialize logger with Debug level for development
    dioxus::logger::init(Level::DEBUG)
        .expect("failed to init logger");

    dioxus::launch(App);
}
```

### Pattern 2: Structured Logging in Components
**What:** Use tracing macros for severity-based logging
**When to use:** Throughout application for debugging, warnings, and errors
**Example:**
```rust
// Source: https://dioxuslabs.com/learn/0.7/guides/utilities/logging/
use tracing::{debug, info, warn, error};

fn App() -> Element {
    use_effect(move || {
        debug!("App component mounted");
        info!("Initializing database...");

        // ... setup code ...

        if let Err(e) = result {
            error!("Database initialization failed: {}", e);
        } else {
            info!("Database initialized successfully");
        }
    });

    rsx! { /* ... */ }
}
```

### Pattern 3: Development Server with Hot-Reload
**What:** Run dx serve for development with automatic rebuilds
**When to use:** During active development for fast iteration
**Example:**
```bash
# Source: https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload/
# Standard hot-reload (RSX and assets)
dx serve

# With experimental Rust hot-patching
dx serve --hotpatch

# Custom port
dx serve --port 8080

# Disable auto-open browser
dx serve --open false
```

### Pattern 4: Panic Hook for Better Error Messages (Optional)
**What:** Install console_error_panic_hook to see formatted panics in console
**When to use:** Development builds for better debugging
**Example:**
```rust
// Source: https://rustwasm.github.io/docs/book/game-of-life/debugging.html
#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();

    dioxus::logger::init(Level::DEBUG)
        .expect("failed to init logger");

    dioxus::launch(App);
}
```

### Anti-Patterns to Avoid
- **Not initializing logger:** Logger must be initialized before dioxus::launch() or early logs are lost
- **Using println! in WASM:** println! doesn't output to browser console; use tracing macros or web_sys::console instead
- **Manual wasm-bindgen invocation:** dx CLI handles this; manual builds create inconsistencies
- **Running dx build instead of dx serve:** dx build is for production bundles; dx serve is for development with hot-reload

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WASM build pipeline | Custom cargo/wasm-bindgen scripts | dx serve / dx build | Handles WASM compilation, JS bindings, asset processing, optimization, and hot-reload |
| Console logging abstraction | Manual web_sys::console wrapper | dioxus::logger | Cross-platform (works on desktop/mobile/web), severity levels, zero-config web integration |
| Development server | Custom HTTP server + file watcher | dx serve | Integrated hot-reload, automatic rebuilds, WASM serving with correct MIME types |
| Source maps for debugging | Custom DWARF tooling | Rely on dx defaults | dx handles debug symbols in dev builds; manual configuration error-prone |

**Key insight:** Dioxus CLI is a complete development environment - attempting to manually orchestrate cargo, wasm-bindgen, and file serving introduces complexity and breaks hot-reload. Use dx as the single entry point for development and builds.

## Common Pitfalls

### Pitfall 1: Logger Not Initialized Before Launch
**What goes wrong:** log::info!() and tracing::debug!() calls produce no output in browser console
**Why it happens:** Dioxus logger needs explicit initialization in main.rs; without it, log facade has no backend
**How to avoid:** Add `dioxus::logger::init(Level::DEBUG).expect("failed to init logger")` before `dioxus::launch()`
**Warning signs:** Existing log::warn!() and log::error!() calls in codebase produce no console output when running app

### Pitfall 2: Wrong Build Command for Development
**What goes wrong:** Using `dx build` or `dx bundle` results in slow rebuilds, no hot-reload, and unnecessary optimization
**Why it happens:** dx build creates production bundles with wasm-opt; dx serve is for development
**How to avoid:** Always use `dx serve` during development; reserve dx bundle for deployment
**Warning signs:** Rebuilds take 30+ seconds; code changes require manual browser refresh; WASM file is very small (over-optimized)

### Pitfall 3: Missing wasm32-unknown-unknown Target
**What goes wrong:** Compilation fails with "can't find crate for `std`" when building for web
**Why it happens:** Rust stdlib for WASM target not installed
**How to avoid:** Verify `rustup target list --installed | grep wasm32-unknown-unknown` shows target; add via `rustup target add wasm32-unknown-unknown` if missing
**Warning signs:** devenv.nix already specifies `targets = ["wasm32-unknown-unknown"]` so this is handled; only relevant if not using devenv

### Pitfall 4: Hot-Reload Limitations Misunderstood
**What goes wrong:** Developer expects full app hot-reload but changes still require rebuild
**Why it happens:** RSX hot-reload only covers UI markup/styles; logic changes in non-RSX code require recompilation
**How to avoid:** Understand what hot-reloads instantly (RSX elements, string attributes, simple literals) vs what needs rebuild (new variables, function calls, imports)
**Warning signs:** Developer keeps editing Rust logic expecting instant updates like JavaScript; need to set expectation that logic changes trigger fast rebuild, not instantaneous hot-patch

### Pitfall 5: CORS Issues with SharedArrayBuffer
**What goes wrong:** Console shows "SharedArrayBuffer is not defined" or similar WASM threading errors
**Why it happens:** sql.js may expect SharedArrayBuffer which requires COOP/COEP headers in modern browsers
**How to avoid:** Use `dx serve --cross-origin-policy` flag to set required headers for development
**Warning signs:** sql.js initialization fails; WASM module load errors mentioning cross-origin isolation

## Code Examples

Verified patterns from official sources:

### Initializing Logger in main.rs
```rust
// Source: https://dioxuslabs.com/learn/0.7/guides/utilities/logging/
use dioxus::prelude::*;
use tracing::Level;

fn main() {
    // Initialize with Debug level for development
    dioxus::logger::init(Level::DEBUG)
        .expect("failed to init logger");

    // Launch app (logger now active)
    dioxus::launch(App);
}
```

### Using Log Levels Appropriately
```rust
// Source: https://dioxuslabs.com/learn/0.7/guides/utilities/logging/
use tracing::{trace, debug, info, warn, error};

fn my_function() {
    trace!("Very detailed trace information");         // Most verbose
    debug!("Debug-level development info");            // Dev mode only
    info!("General informational message");            // Dev + production
    warn!("Warning condition detected");               // Always shown
    error!("Error occurred: {}", error_message);       // Always shown
}
```

### Development Server Commands
```bash
# Source: https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0
# Standard development with hot-reload
dx serve

# With Rust hot-patching (experimental)
dx serve --hotpatch

# Enable CORS headers for SharedArrayBuffer
dx serve --cross-origin-policy

# Custom port and disable auto-open
dx serve --port 3000 --open false

# With verbose logging for troubleshooting
dx serve --verbose
```

### Checking Environment Setup
```bash
# Verify Dioxus CLI version
dx --version
# Should show: dioxus 0.7.2 (or later)

# Verify Rust toolchain
rustc --version
# Should show: rustc 1.92.0 or compatible

# Verify WASM target installed
rustup target list --installed | grep wasm32
# Should show: wasm32-unknown-unknown

# Verify dx can parse project
dx doctor
# Checks Dioxus.toml, Cargo.toml, and toolchain
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| trunk + wasm-pack | dx serve with hot-reload | Dioxus 0.5+ | Unified CLI replaces two separate tools; better DX with integrated hot-reload |
| Manual console_log crate setup | dioxus::logger::init() | Dioxus 0.7.0 | Zero-config logging; automatically uses tracing-wasm on web platform |
| RSX hot-reload only | RSX + Rust hot-patching | Dioxus 0.7.0 | Experimental --hotpatch flag enables hot-reloading of Rust logic via Subsecond |
| println! debugging | tracing macros with levels | Current best practice | Structured logging with severity levels; cross-platform (not just web) |

**Deprecated/outdated:**
- **trunk serve:** Dioxus projects should use dx serve instead; trunk is generic WASM tool, dx is Dioxus-optimized
- **wasm-pack build --target web:** dx handles this internally; manual wasm-pack adds complexity
- **println! for logging:** Doesn't work in WASM; use tracing macros or web_sys::console

## Open Questions

1. **Does existing codebase need console_error_panic_hook?**
   - What we know: Not currently in Cargo.toml dependencies
   - What's unclear: Whether panic messages are readable without it; depends on browser DevTools WASM support
   - Recommendation: Try without first; add only if panics show as cryptic "RuntimeError: unreachable" instead of Rust panic messages

2. **Is --cross-origin-policy needed for sql.js?**
   - What we know: sql.js is loaded via script tag in index.html; uses public/sql-wasm.wasm
   - What's unclear: Whether current sql.js build requires SharedArrayBuffer (which needs COOP/COEP headers)
   - Recommendation: Test with standard `dx serve` first; add --cross-origin-policy flag only if WASM load fails with cross-origin errors

3. **Should hot-patching (--hotpatch) be used for this phase?**
   - What we know: It's experimental in Dioxus 0.7; works for "tip crate" only (not dependencies)
   - What's unclear: Stability for debugging file picker issues; may introduce unexpected behavior
   - Recommendation: Start with standard hot-reload; use --hotpatch only if development iteration speed becomes bottleneck

## Validation Architecture

> Validation enabled: workflow.nyquist_validation is true in .planning/config.json

### Test Framework
| Property | Value |
|----------|-------|
| Framework | wasm-bindgen-test 0.3 |
| Config file | none - configured in test files via wasm_bindgen_test_configure! |
| Quick run command | `wasm-pack test --headless --chrome` (requires wasm-pack installation) |
| Full suite command | `wasm-pack test --headless --chrome` (same - all tests run together) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEV-01 | Development server serves app in browser | manual | N/A - run `dx serve` and verify http://localhost:8080 loads | N/A |
| DEV-02 | Browser console shows logs | manual | N/A - open DevTools console, verify tracing::info!/debug! messages appear | N/A |
| DEV-03 | WASM compilation succeeds | unit | `cargo check --target wasm32-unknown-unknown` | ✅ Cargo.toml exists |
| DEV-04 | Hot reload responds to code changes | manual | N/A - edit RSX in src/app.rs, verify browser updates without full refresh | N/A |

**Note:** Phase 1 requirements are primarily environmental setup and cannot be easily unit-tested. Validation is manual verification that dev server runs, console logs appear, WASM compiles, and hot-reload works.

### Sampling Rate
- **Per task commit:** `cargo check --target wasm32-unknown-unknown` (fast syntax check)
- **Per wave merge:** `dx build` (full compilation without optimization)
- **Phase gate:** Manual verification of all four success criteria before /gsd:verify-work

### Wave 0 Gaps
- [ ] No automated tests for DEV-01 through DEV-04 - all are environmental/manual verification
- [ ] Consider adding smoke test: compile + serve + basic health check in CI/CD (out of scope for this phase)

**Existing test infrastructure:** Project already has wasm-bindgen-test setup with 29+ existing tests in db_tests.rs, file_system_tests.rs, and validation.rs. Test execution requires wasm-pack (currently not in devenv.nix but mentioned in TESTING.md).

## Sources

### Primary (HIGH confidence)
- [Dioxus 0.7 Logging Guide](https://dioxuslabs.com/learn/0.7/guides/utilities/logging/) - Official documentation on dioxus::logger setup and tracing macros
- [Dioxus 0.7 Hot-Reload Guide](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload/) - Official documentation on RSX hot-reload and Rust hot-patching
- [Dioxus v0.7.0 Release](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0) - Official release notes for 0.7.0 features
- [wasm-bindgen Console Logging](https://rustwasm.github.io/docs/wasm-bindgen/examples/console-log.html) - Official wasm-bindgen documentation on console access
- [Rust WASM Debugging Guide](https://rustwasm.github.io/docs/book/game-of-life/debugging.html) - Official Rust-WASM book on debugging techniques
- Codebase analysis: Cargo.toml, devenv.nix, existing src/ structure - verified via Read tool

### Secondary (MEDIUM confidence)
- [Dioxus 0.7 Hot Reload GitHub Discussion](https://github.com/DioxusLabs/dioxus/issues/3653) - Community discussion on logging and debugging
- [WASM Debugging in Chrome](https://users.rust-lang.org/t/getting-raw-wasm-debugging-working-nicely-in-chrome-devtools/94646) - Community best practices for browser debugging

### Tertiary (LOW confidence)
- None - all research backed by official documentation or verified codebase inspection

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in Cargo.toml and devenv.nix; versions verified
- Architecture: HIGH - Patterns from official Dioxus 0.7 documentation with verified code examples
- Pitfalls: MEDIUM - Based on official docs and community discussions; some project-specific issues may emerge

**Research date:** 2026-02-25
**Valid until:** 2026-03-27 (30 days) - Dioxus 0.7 is stable; no major changes expected in this timeframe
