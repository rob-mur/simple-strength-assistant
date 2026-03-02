# Phase 4: Tab Navigation Foundation - Context

**Gathered:** 2026-03-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable navigation between Workout and Library tabs without losing active workout session state. This delivers the navigation structure for the v1.1 Exercise Library milestone (requirements LIB-01, LIB-02).

</domain>

<decisions>
## Implementation Decisions

### Testing Strategy
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

</decisions>

<specifics>
## Specific Ideas

- Follow the TDD/BDD pattern established in the tape measure component (phases 04-01, 04-02, 04-03)
- Keep Playwright test suite minimal to avoid slow test runs
- Use tags to organize scenarios by test type rather than splitting into separate repositories

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope

</deferred>

---

*Phase: 04-tab-navigation-foundation*
*Context gathered: 2026-03-02*
