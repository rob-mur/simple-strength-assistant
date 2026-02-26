# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — File Picker Fix

**Shipped:** 2026-02-26
**Phases:** 3 | **Plans:** 6

### What Was Built
- Robust File System Access API integration with user-gesture triggering.
- Automatic permission state management for cached file handles.
- Cross-browser LocalStorage fallback for non-supported environments.
- Polished Error UI with recovery steps and PWA installability on mobile.

### What Worked
- **Inline initialization**: Eliminating string-matching on error messages by continuing initialization inline after user-triggered file picking was a huge reliability win.
- **Signal-based reactivity**: Moving to Dioxus 0.7 Signals simplified state management across the app.

### What Was Inefficient
- **Initial auto-prompting**: Wasted time trying to make `showSaveFilePicker` work on page load before realizing it's a hard browser requirement to have a user gesture.

### Patterns Established
- **User-triggered async flows**: Always wrap restricted APIs (File System, Permissions) in a user gesture button.
- **Detailed console logging**: Using `[FileSystem]` and `[DB Init]` prefixes for all infrastructure logs made debugging much easier.

### Key Lessons
1.  **Transient User Activation**: The File System Access API requires a user gesture. Never try to auto-prompt for file access on page load.
2.  **Vercel Deployment Constraints**: Manifest files for PWAs on protected Vercel deployments need `crossorigin="use-credentials"` to be fetchable.
3.  **Dioxus 0.7 Signal Patterns**: Using signals for global state (especially in `Copy` structs) provides clean, reactive updates without manual re-rendering logic.

### Cost Observations
- Model mix: 100% agent (using available tools).
- Notable: Very high efficiency in Phase 2 due to precise error logging.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 1 | 3 | Initial milestone completion using GSD workflows. |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | 7 (manual) | N/A | Added `tracing 0.1` |

### Top Lessons (Verified Across Milestones)

1.  Prioritize infrastructure reliability (persistence) before building features.
2.  Clear, actionable error messages reduce support overhead and user frustration.
