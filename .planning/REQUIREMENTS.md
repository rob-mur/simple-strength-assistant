# Requirements: Simple Strength Assistant

**Defined:** 2026-03-03
**Core Value:** Users must be able to reliably persist their workout data to a file they control on their device.

## v1.2 Requirements

Minimum Weight milestone - ensure actionable weight suggestions by replacing 'starting weight' with a configurable 'minimum weight'.

### Exercise Configuration

- [ ] **CONF-01**: User can define a minimum weight for each exercise (e.g., 20kg for barbell)
- [ ] **CONF-02**: Minimum weight input validates as a positive numerical value and defaults to 0
- [ ] **CONF-03**: Application no longer provides UI or logic for 'Starting Weight'

### Suggestion Engine

- [ ] **SUGG-01**: Session suggestion uses most recent recorded weight from previous session of the exercise
- [ ] **SUGG-02**: Session suggestion falls back to defined Minimum Weight if no previous records exist

## v1.1 Requirements (Completed)

- [x] **LIB-01**: User can view Exercise Library tab in the main interface
- [x] **LIB-02**: User can switch between Workout and Library tabs without losing active session
- [x] **LIB-03**: User can see all exercises they've created in a list view
- [x] **LIB-04**: User can search exercises by name with instant filtering
- [x] **LIB-05**: User sees exercise type indicator (weighted vs bodyweight) for each exercise
- [x] **LIB-06**: User sees clear empty state message when no exercises exist

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Exercise Metadata

- **LIB-07**: User can see last performed date for each exercise
- **LIB-08**: User can see total sessions count for each exercise

### Exercise Management

- **LIB-09**: User can archive exercises to hide from library
- **LIB-10**: User can edit exercise name and settings

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature                                 | Reason                                                                           |
| --------------------------------------- | -------------------------------------------------------------------------------- |
| Pre-populated exercise database         | User builds personal library through workouts; avoids naming conflicts and bloat |
| Exercise categorization by muscle group | Adds complexity without validation; users already know exercise names            |
| Delete exercises                        | Archive instead to preserve workout history and prevent orphaned data            |
| Exercise video/image tutorials          | Massive storage overhead, scope creep; user can Google form cues                 |
| Workout prescription from library       | Belongs in future prescription milestone, not library browsing                   |
| Workout prescription based on history   | Complex feature reserved for a future milestone (only recent/min weight for now) |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase   | Status  |
| ----------- | ------- | ------- |
| CONF-01     | Phase 7 | Pending |
| CONF-02     | Phase 7 | Pending |
| CONF-03     | Phase 7 | Pending |
| SUGG-01     | Phase 7 | Pending |
| SUGG-02     | Phase 7 | Pending |

**Coverage:**

- v1.2 requirements: 5 total
- Mapped to phases: 5
- Unmapped: 0 ✓

---

_Requirements defined: 2026-03-03_
_Last updated: 2026-03-03 after requirement definition_
