# Phase 05: Exercise List & Search - Research

**Researched:** 2026-03-02
**Domain:** UI Component State Management & BDD Testing in Dioxus
**Confidence:** HIGH

## Summary

This phase implements a list view and instant text-based filtering within the Dioxus `LibraryView` component, adhering to the BDD testing approach established in Phase 04. State management relies on Dioxus `use_signal` for tracking the search input, while the exercise data comes from the application's global state (`WorkoutState` or `Database`). The UI will dynamically filter the displayed exercises and present empty states when applicable. 

**Primary recommendation:** Use Dioxus `use_signal` for localized search input state and perform derived filtering during render based on the global exercise list. Follow the established BDD pattern with `cucumber-rust` for defining scenarios before implementation.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- BDD-first approach: Generate feature files first, then implement to make them pass
- Use separate feature files: one for exercise list display, another for search filtering behavior
- Continue using cucumber-rust as test runner (consistent with Phase 4 tab navigation tests)
- Tag scenarios to distinguish test levels: 
  - `@e2e` for Playwright tests (slowest, run 2-3 critical user flows)
  - `@unit` for mocked component behavior tests (broader coverage, faster)
- Playwright scenarios should cover:
  - Exercise list display with type badges
  - Search filtering with instant results
  - Empty state display (no exercises, no search results)
- Mocked tests cover component-level details and edge cases

### Claude's Discretion
- Exercise list UI layout (card vs list view, spacing, typography)
- Search box placement and styling
- Exercise type badge design (colors, icons, text)
- Empty state message styling
- Search filtering algorithm implementation details
- List scrolling behavior and performance optimizations

### Deferred Ideas
None - context stays within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LIB-03 | User can see all exercises they've created in a list view | Dioxus list rendering using `rsx!` with map over exercises |
| LIB-04 | User can search exercises by name with instant filtering | Derived state combining search `Signal<String>` and global exercises |
| LIB-05 | User sees exercise type indicator | UI badge component mapped to exercise metadata (`bodyweight`/`weighted`) |
| LIB-06 | User sees clear empty state message | Conditional rendering (`if exercises.is_empty()`) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| dioxus | 0.7.2 | UI framework | Project standard UI framework |
| cucumber | 0.21 | BDD testing | Project standard for BDD test execution |
| playwright | ^1.x | E2E testing | Project standard for browser automation |

## Architecture Patterns

### BDD Feature File Structure
```gherkin
Feature: Exercise List Display
  Scenario: Viewing the exercise list
  Scenario: Empty state for new users
```

### Recommended Dioxus Pattern
**What:** Local state for search string, derived filtering during render
**When to use:** For fast, client-side list filtering without expensive re-computations
**Example:**
```rust
let mut search_query = use_signal(|| String::new());
let exercises = use_context::<Signal<WorkoutState>>();

let filtered_exercises = exercises.read().get_exercises().into_iter()
    .filter(|e| e.name.to_lowercase().contains(&search_query.read().to_lowercase()))
    .collect::<Vec<_>>();
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fuzzy search | Complex fuzzy matching algorithm | Simple `String::contains(lowercase)` | MVP requirement is just "instant text filtering", keep it simple and fast |
| Debouncing | Custom debounce timer | Instant state updates | The dataset is small (local exercises), so standard React/Dioxus fast updates handle it fine. No debouncing needed for MVP. |

## Common Pitfalls

### Pitfall 1: Case-Sensitive Search
**What goes wrong:** User types "bench" and doesn't see "Bench Press"
**Why it happens:** Default Rust string matching is case-sensitive
**How to avoid:** Normalize both query and target to lowercase before comparison

### Pitfall 2: Too Many E2E Tests
**What goes wrong:** CI runtime explodes because every BDD scenario runs through Playwright
**Why it happens:** Mapping all scenarios to the E2E runner instead of using `@unit` where appropriate
**How to avoid:** Only map critical flows to Playwright (`@e2e`), use mocked behavior tests for the rest (`@unit`).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cucumber-rust + playwright |
| Config file | `tests/e2e/steps/mod.rs` |
| Quick run command | `cargo test --test "*_bdd"` |
| Full suite command | `./scripts/ci-test.sh` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LIB-03 | See exercises | E2E | `npm run playwright test` | ❌ Wave 0 |
| LIB-04 | Search filter | E2E | `npm run playwright test` | ❌ Wave 0 |
| LIB-05 | Exercise type | Unit | `cargo test` | ❌ Wave 0 |
| LIB-06 | Empty state | Unit | `cargo test` | ❌ Wave 0 |

### Wave 0 Gaps
- [ ] `tests/features/exercise_list_display.feature`
- [ ] `tests/features/exercise_search_filter.feature`
- [ ] `tests/exercise_list_bdd.rs` (cucumber runner)
- [ ] Playwright step definitions in TS

