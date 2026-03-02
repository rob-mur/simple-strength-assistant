# Phase 4: Tab Navigation Foundation - Research

**Researched:** 2026-03-02
**Domain:** Tab navigation, state preservation, Dioxus context management
**Confidence:** HIGH

## Summary

Phase 4 implements tab navigation between Workout and Library views without losing active workout session state. The technical challenge is preserving state across view switches and browser refreshes without using full routing infrastructure.

The recommended approach leverages Dioxus's existing context provider system (already in use for `WorkoutState`), conditional rendering to switch between tab content, and localStorage for tab selection persistence. This avoids introducing the Dioxus Router dependency, which would be overkill for a simple two-tab interface and could complicate state management.

The project already uses Dioxus 0.7.2, DaisyUI 4.12 for styling, and playwright-bdd for E2E testing. The existing WorkoutState context provider pattern established in v1.0 provides the foundation for state preservation.

**Primary recommendation:** Use conditional rendering with a Signal to track active tab, persist tab selection to localStorage with gloo-storage, and rely on existing WorkoutState context provider to maintain session state across tab switches.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- BDD-first approach: Generate feature files first, then implement to make them pass
- Use separate feature files: one for tab navigation UI, another for session state preservation logic
- Continue using cucumber-rust as test runner (consistent with existing tape measure tests)
- Tag scenarios to distinguish test levels:
  - `@e2e` for Playwright tests (slowest, run 2-3 critical user flows)
  - `@unit` for mocked component behavior tests (broader coverage, faster)
- Playwright scenarios should cover:
  - Tab switching between Workout and Library
  - Active workout session state persistence during tab changes
  - Tab selection persistence on browser refresh
- Mocked tests cover component-level details and edge cases

### Claude's Discretion
- Tab UI presentation (bottom bar vs top bar, visual styling, active indication)
- Tab switching behavior (animations, gestures, haptic feedback)
- Specific state preservation implementation details
- Library tab placeholder content design

### Deferred Ideas (OUT OF SCOPE)
None - discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LIB-01 | User can view Exercise Library tab in the main interface | DaisyUI tab components + conditional rendering pattern |
| LIB-02 | User can switch between Workout and Library tabs without losing active session | Dioxus use_context + Signal for tab state + existing WorkoutState persistence |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Dioxus | 0.7.2 | React-like UI framework for Rust/WASM | Already in use, provides Signal and context hooks |
| DaisyUI | 4.12.14 | Tailwind CSS component library | Already in use, provides ready-made tab components |
| gloo-storage | 0.3 | Browser localStorage/sessionStorage wrapper | Already in dependencies, provides type-safe storage |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| playwright-bdd | 8.4.2 | BDD testing with Playwright | Already in use for E2E tests |
| web-sys | 0.3 | Raw WASM bindings to Web APIs | Already in use for file system APIs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Conditional rendering | Dioxus Router | Router is overkill for 2 tabs, adds complexity and bundle size (~50KB), complicates state preservation |
| gloo-storage | Direct localStorage via web-sys | gloo-storage provides serde integration and error handling, reduces boilerplate |
| Context provider | Global Signal | Context provider already established pattern in codebase, cleaner scoping |

**Installation:**
No new dependencies required - all libraries already in project.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── app.rs                  # Root component with tab state management
├── components/
│   ├── mod.rs
│   ├── workout_view.rs     # Extract current workout UI (NEW)
│   ├── library_view.rs     # Library placeholder (NEW)
│   └── tab_bar.rs          # Tab navigation UI (NEW)
├── state/
│   ├── mod.rs
│   ├── workout_state.rs    # Existing - no changes needed
│   └── tab_state.rs        # Tab persistence logic (NEW)
```

### Pattern 1: Conditional View Rendering with Signal
**What:** Use a Signal to track active tab and conditionally render view components
**When to use:** When you need client-side navigation without URL changes
**Example:**
```rust
// Source: Dioxus 0.7 docs + project patterns
#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Workout,
    Library,
}

#[component]
pub fn App() -> Element {
    let workout_state = use_context_provider(WorkoutState::new);
    let mut active_tab = use_signal(|| Tab::Workout);

    // Load persisted tab on mount
    use_effect(move || {
        if let Ok(saved_tab) = LocalStorage::get("active_tab") {
            active_tab.set(saved_tab);
        }
    });

    rsx! {
        div {
            class: "flex flex-col min-h-screen bg-base-200",

            // Header (existing)
            header { /* ... */ }

            // Main content - conditional rendering
            main {
                class: "flex-1 container mx-auto p-4",
                match active_tab() {
                    Tab::Workout => rsx! { WorkoutView {} },
                    Tab::Library => rsx! { LibraryView {} },
                }
            }

            // Tab bar at bottom
            TabBar {
                active_tab: active_tab(),
                on_change: move |tab| {
                    active_tab.set(tab);
                    let _ = LocalStorage::set("active_tab", &tab);
                }
            }
        }
    }
}
```

### Pattern 2: Context-Based State Preservation
**What:** Use existing use_context_provider to maintain state across view switches
**When to use:** When multiple views need access to shared state
**Example:**
```rust
// Source: Existing project pattern (src/app.rs lines 71-72)
// WorkoutState already provided at root level
let workout_state = use_context_provider(WorkoutState::new);

// Child components access without prop drilling
#[component]
fn WorkoutView() -> Element {
    let workout_state = use_context::<WorkoutState>();
    // State persists when switching to Library tab and back
    rsx! { /* render with workout_state */ }
}
```

### Pattern 3: localStorage Tab Persistence
**What:** Save/restore active tab selection to survive browser refresh
**When to use:** For preserving navigation state across page reloads
**Example:**
```rust
// Source: gloo-storage docs
use gloo_storage::{LocalStorage, Storage};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
enum Tab {
    Workout,
    Library,
}

// Save on tab change
let _ = LocalStorage::set("active_tab", &Tab::Library);

// Load on mount
let saved_tab: Tab = LocalStorage::get("active_tab")
    .unwrap_or(Tab::Workout);
```

### Anti-Patterns to Avoid
- **Don't use full Router for two tabs:** Dioxus Router adds ~50KB and URL management complexity that isn't needed. Conditional rendering is simpler and faster.
- **Don't recreate WorkoutState:** The existing context provider pattern already handles state preservation. New tab state should be separate Signal, not part of WorkoutState.
- **Don't use session storage for tab selection:** Use localStorage instead. SessionStorage clears on browser close, localStorage persists indefinitely (matches user expectation for navigation state).
- **Don't unmount/remount WorkoutView:** Keep the component mounted but hidden (display: none) to preserve internal component state like animation frames. Actually, conditional rendering is fine because state lives in WorkoutState context, not component local state.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tab UI styling | Custom CSS for tabs, active states, hover effects | DaisyUI `.tabs`, `.tab`, `.tab-active` classes | DaisyUI handles accessibility (aria-labels, roles), responsive sizing, theme integration, and focus states automatically |
| localStorage serialization | Manual JSON.stringify/parse with error handling | gloo-storage with serde | Type-safe, handles serialization errors, provides Result types, already in dependencies |
| Browser context state | Custom localStorage wrapper for test mode | Existing InMemoryStorage pattern | Project already has StorageBackend trait for test/production switching (src/state/storage.rs) |

**Key insight:** Tab navigation is UI state, not domain state. Keep it separate from WorkoutState. The complexity is in preserving *other* state across tab switches, not in the tabs themselves.

## Common Pitfalls

### Pitfall 1: Using Dioxus Router for Simple Tab Navigation
**What goes wrong:** Router introduces URL management, history entries, and async route matching for a use case that doesn't need it
**Why it happens:** Developers default to "navigation = router" pattern from web frameworks
**How to avoid:** Only use Router when you need: URL-based navigation, deep linking, browser back/forward integration, or >5 distinct routes
**Warning signs:** If you're writing `Route::Workout {}` and `Route::Library {}` enum variants, you're over-engineering

### Pitfall 2: Re-mounting Components on Tab Switch
**What goes wrong:** Component state resets, timers restart, animations glitch, scroll position lost
**Why it happens:** Using `if` statements instead of `match` for conditional rendering, or not understanding Dioxus's reconciliation
**How to avoid:** Use consistent `match` statements. Dioxus preserves component identity when switching between branches if component types match. Better yet, rely on context state (WorkoutState) not component local state.
**Warning signs:** User reports "timer resets when switching tabs" or "lost my place"

### Pitfall 3: Not Testing State Persistence
**What goes wrong:** State appears to work in development but fails after browser refresh or across sessions
**Why it happens:** localStorage writes can fail silently (quota exceeded, privacy mode), deserialization errors not handled
**How to avoid:** BDD scenarios must include "User refreshes browser → active tab and workout state restored". Mock localStorage failures in unit tests.
**Warning signs:** E2E tests pass but manual testing shows state loss

### Pitfall 4: Forgetting Accessibility for Tabs
**What goes wrong:** Keyboard navigation broken, screen readers announce incorrectly, ARIA attributes missing
**Why it happens:** Implementing tabs as divs with onClick instead of semantic elements
**How to avoid:** Use DaisyUI tab components which include `role="tablist"`, `role="tab"`, `aria-selected`, and keyboard navigation. Verify with keyboard-only testing.
**Warning signs:** Can't navigate tabs with arrow keys, screen reader says "clickable" instead of "tab"

## Code Examples

Verified patterns from official sources and existing project code:

### Tab Component with DaisyUI
```rust
// Source: DaisyUI docs + Dioxus patterns
#[component]
fn TabBar(active_tab: Tab, on_change: EventHandler<Tab>) -> Element {
    rsx! {
        div {
            role: "tablist",
            class: "tabs tabs-boxed fixed bottom-0 left-0 right-0 bg-base-100 shadow-lg",

            button {
                role: "tab",
                class: if active_tab == Tab::Workout { "tab tab-active" } else { "tab" },
                onclick: move |_| on_change.call(Tab::Workout),
                "Workout"
            }

            button {
                role: "tab",
                class: if active_tab == Tab::Library { "tab tab-active" } else { "tab" },
                onclick: move |_| on_change.call(Tab::Library),
                "Library"
            }
        }
    }
}
```

### Signal with localStorage Persistence
```rust
// Source: Dioxus use_signal docs + gloo-storage docs
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
enum Tab {
    Workout,
    Library,
}

#[component]
pub fn App() -> Element {
    let mut active_tab = use_signal(|| {
        LocalStorage::get("active_tab").unwrap_or(Tab::Workout)
    });

    // Persist on change
    let handle_tab_change = move |tab: Tab| {
        active_tab.set(tab);
        let _ = LocalStorage::set("active_tab", &tab);
    };

    rsx! {
        // ... render with active_tab() and handle_tab_change
    }
}
```

### Context Provider State Access (Existing Pattern)
```rust
// Source: Project src/app.rs line 71
// Root level - already exists
let workout_state = use_context_provider(WorkoutState::new);

// Child component - how WorkoutView and LibraryView will access state
#[component]
fn WorkoutView() -> Element {
    let workout_state = use_context::<WorkoutState>();

    // All workout state persists across tab switches
    // because context lives at root level above tabs
    rsx! { /* render workout UI */ }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Page refreshes for navigation | Client-side conditional rendering | ~2015 (SPAs) | Instant navigation, state preservation |
| Prop drilling state through layers | Context providers | Dioxus 0.5+ | Simpler component trees, no boilerplate |
| Manual localStorage JSON | Type-safe storage libraries | 2020+ | Fewer runtime errors, better DX |
| ARIA roles as afterthought | Semantic HTML + built-in accessibility | WCAG 2.1+ (2018) | Better screen reader support, keyboard nav |

**Deprecated/outdated:**
- `use_shared_state` hook: Replaced by `use_context` in Dioxus 0.5+, simpler API
- Manually managing tab state in URL: Only needed for deep linking, localStorage sufficient for simple cases
- Full page refreshes for tab changes: SPA patterns now standard, unnecessary server round-trips

## Open Questions

1. **Should Library tab be lazy-loaded?**
   - What we know: Library content not needed until user switches tabs
   - What's unclear: Whether lazy loading adds meaningful performance benefit for placeholder content
   - Recommendation: Start with eager rendering (both tabs in DOM, one hidden). Optimize to lazy loading if Library grows complex (Phase 5+).

2. **Should tab state be part of WorkoutState?**
   - What we know: WorkoutState manages domain state (exercises, sets, timer), tab is UI state
   - What's unclear: Whether keeping them separate or unified is cleaner
   - Recommendation: Keep separate. Tab state is navigation (UI concern), workout state is domain logic. Separation of concerns prevents unrelated changes from coupling.

3. **How to handle deep linking to Library in future?**
   - What we know: Phase 4 doesn't require URL-based navigation
   - What's unclear: If future phases need sharable Library URLs (e.g., "show me this exercise")
   - Recommendation: Current conditional rendering approach is compatible with future Router addition. Can wrap in Router later without refactoring tab logic.

## Validation Architecture

> Note: workflow.nyquist_validation not found in config - this section skipped per instructions

## Sources

### Primary (HIGH confidence)
- [Dioxus 0.7 Navigation Docs](https://dioxuslabs.com/learn/0.7/essentials/router/navigation/) - Link component and navigator hooks
- [Dioxus 0.7 Context Docs](https://dioxuslabs.com/learn/0.7/essentials/basics/context/) - use_context_provider patterns
- [DaisyUI Tab Components](https://daisyui.com/components/tab/) - Tab styling and accessibility
- [gloo-storage API Docs](https://docs.rs/gloo-storage/latest/gloo_storage/) - localStorage wrapper patterns
- Project source code (src/app.rs, Cargo.toml, package.json) - Existing architecture patterns

### Secondary (MEDIUM confidence)
- [Dioxus Router Documentation](https://dioxuslabs.com/learn/0.7/essentials/router/routes/) - Router alternatives comparison
- [ARIA Tab Role (MDN)](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/tab_role) - Accessibility requirements
- [Material Design Bottom Navigation](https://m2.material.io/components/bottom-navigation) - Mobile tab bar patterns
- [W3C ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/) - Tab navigation patterns
- [Mobile Navigation Patterns 2026](https://phone-simulator.com/blog/mobile-navigation-patterns-in-2026) - Bottom nav best practices

### Secondary (MEDIUM confidence) - Community & Ecosystem
- [Reactivity and State Management (DeepWiki)](https://deepwiki.com/DioxusLabs/dioxus/3-reactivity-and-state-management) - Signal patterns
- [playwright-bdd GitHub](https://github.com/vitalets/playwright-bdd) - BDD testing patterns
- [Dioxus Router Discussions](https://github.com/DioxusLabs/dioxus/discussions/2368) - Navigation state handling

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in project, versions verified from Cargo.toml and package.json
- Architecture: HIGH - Patterns verified from official Dioxus docs and existing project code
- Pitfalls: MEDIUM - Based on React/web framework patterns and Dioxus community discussions, some extrapolation
- Testing: MEDIUM - playwright-bdd patterns verified, but Dioxus-specific test strategies partially inferred

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (30 days - Dioxus 0.7 is stable, tab patterns are mature)

**Key findings:**
1. No new dependencies required - all tools already in project
2. Conditional rendering + Signal preferred over Router for 2-tab case
3. Existing WorkoutState context provider handles state preservation automatically
4. DaisyUI provides accessible tab components, reducing custom implementation
5. gloo-storage already available for localStorage persistence
