# Requirements: Simple Strength Assistant

**Defined:** 2026-02-25
**Core Value:** Users must be able to reliably persist their workout data to a file they control on their device

## v1 Requirements

### Database Setup

- [ ] **DB-01**: File picker dialog appears when user needs to select database location
- [x] **DB-02**: User can successfully select a .sqlite or .db file from their filesystem
- [ ] **DB-03**: Selected file handle persists across browser sessions (via IndexedDB)
- [x] **DB-04**: User can grant File System Access API permissions when prompted
- [ ] **DB-05**: Database initialization completes successfully after file selection
- [ ] **DB-06**: LocalStorage fallback works when File System Access API unavailable

### Development Environment

- [x] **DEV-01**: Development server runs and serves app in browser
- [x] **DEV-02**: Browser console logs are accessible for debugging
- [x] **DEV-03**: WASM compilation succeeds without errors
- [x] **DEV-04**: Hot reload works for Rust code changes

### Error Handling

- [ ] **ERR-01**: File picker errors are logged to console with clear messages
- [x] **ERR-02**: Permission denied shows user-friendly error message
- [ ] **ERR-03**: File format validation errors are surfaced to user
- [ ] **ERR-04**: WASM-JS boundary errors include stack traces

## v2 Requirements

### Data Management

- **DATA-01**: User can export database to JSON format
- **DATA-02**: User can import data from previous exports
- **DATA-03**: User can vacuum/optimize database file

### Advanced Features

- **FEAT-01**: Workout prescription based on history (suggest weight/reps)
- **FEAT-02**: Historical data view showing past sessions
- **FEAT-03**: Progress tracking and visualization
- **FEAT-04**: Undo/edit logged sets

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cloud sync | Offline-first philosophy, user owns data locally |
| Multi-device | Requires cloud infrastructure, conflicts with offline-first |
| Social features | Not relevant to personal workout tracking |
| Workout programs library | Defer until prescription system exists |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DEV-01 | Phase 1 | Complete |
| DEV-02 | Phase 1 | Complete |
| DEV-03 | Phase 1 | Complete |
| DEV-04 | Phase 1 | Complete |
| DB-01 | Phase 2 | Pending |
| DB-02 | Phase 2 | Complete |
| DB-04 | Phase 2 | Complete |
| ERR-01 | Phase 2 | Pending |
| ERR-02 | Phase 2 | Complete |
| ERR-04 | Phase 2 | Pending |
| DB-03 | Phase 3 | Pending |
| DB-05 | Phase 3 | Pending |
| DB-06 | Phase 3 | Pending |
| ERR-03 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-25*
*Last updated: 2026-02-25 after initial definition*
