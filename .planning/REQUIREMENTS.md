# Requirements: Tactile Training Experience (v1.1)

**Defined:** 2026-02-27
**Core Value:** Recording sets with zero typing friction.

## v1 Requirements

Requirements for the tactile, "no-typing" interface.

### Tape Measure (Reps & Weight)

- [ ] **TAPE-01**: User can swipe a horizontal tape measure to adjust weight.
- [ ] **TAPE-02**: User can swipe a horizontal tape measure to adjust reps.
- [ ] **TAPE-03**: Tape measure snaps to discrete increments (e.g., 0.5kg for weight, 1 for reps).
- [ ] **TAPE-04**: User can click on any visible tick mark on the tape measure to jump to that value (desktop support).
- [ ] **TAPE-05**: Component prevents browser scrolling while the tape is being swiped (`touch-action: none`).

### RPE Slider

- [ ] **RPE-01**: User can adjust RPE using a horizontal slider from 1 to 10.
- [ ] **RPE-02**: RPE slider snaps to 0.5 increments.
- [ ] **RPE-03**: Current RPE value is displayed prominently above the slider.

### Big Step Controls

- [ ] **STEP-01**: User can click "Big Step" buttons (e.g., ±5, ±10, ±25) to jump larger distances on the weight tape measure.
- [ ] **STEP-02**: User can click "Small Step" buttons (e.g., ±1) to jump on the reps tape measure.

### UI Integration

- [ ] **INT-01**: The new tactile components replace existing number inputs in the set recording row.
- [ ] **INT-02**: Tactile components are sized appropriately for thumb interaction on mobile devices.
- [ ] **INT-03**: Current values are synchronized with the application's global state (`WorkoutState`).

## v2 Requirements (Future)

- **PRES-01**: Tape measure defaults to predicted/last set values.
- **HAPT-01**: Vibration feedback on set completion.
- **MOMENT-01**: Momentum scrolling for the tape measure.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Native Keyboard Entry | The goal is specifically to eliminate the keyboard friction. |
| Progress Charts | Moved to v1.2 milestone to focus on input usability first. |
| Voice Entry | High complexity, not requested. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TAPE-01 | Phase 4 | Pending |
| TAPE-02 | Phase 4 | Pending |
| TAPE-03 | Phase 4 | Pending |
| TAPE-04 | Phase 4 | Pending |
| TAPE-05 | Phase 4 | Pending |
| RPE-01 | Phase 5 | Pending |
| RPE-02 | Phase 5 | Pending |
| RPE-03 | Phase 5 | Pending |
| STEP-01 | Phase 6 | Complete |
| STEP-02 | Phase 6 | Complete |
| INT-01 | Phase 7 | Pending |
| INT-02 | Phase 7 | Pending |
| INT-03 | Phase 7 | Pending |

**Coverage:**
- v1 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-27*
*Last updated: 2026-02-27 after initial definition*
