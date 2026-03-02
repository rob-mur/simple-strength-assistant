# Phase 5: Exercise List & Search - Context

**Gathered:** 2026-03-02
**Status:** Ready for planning
**Source:** Derived from Phase 4 BDD approach

<domain>
## Phase Boundary

Enable users to browse all created exercises and search by name with instant filtering. This delivers the exercise browsing functionality for the v1.1 Exercise Library milestone (requirements LIB-03, LIB-04, LIB-05, LIB-06).

</domain>

<decisions>
## Implementation Decisions

### Testing Strategy
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

</decisions>

<specifics>
## Specific Ideas

- Follow the TDD/BDD pattern established in Phase 4 (tab navigation) and earlier tape measure phases
- Keep Playwright test suite minimal to avoid slow test runs
- Use tags to organize scenarios by test type rather than splitting into separate repositories
- Search should filter instantly as user types (no debouncing required for MVP)

</specifics>

<deferred>
## Deferred Ideas

None - context stays within phase scope

</deferred>

---

*Phase: 05-exercise-list-and-search*
*Context gathered: 2026-03-02*
