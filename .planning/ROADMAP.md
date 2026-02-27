# Roadmap: Tactile Training Experience (v1.1)

**Milestone:** v1.1
**Goal:** Implement a tactile, "no-typing" interface for recording workout sets.

## Proposed Phases

**4 phases** | **13 requirements mapped** | All covered ✓

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 4 | **Tape Measure** | Core implementation of swipeable inputs for Reps & Weight. | TAPE-[01-05] | 2 |
| 5 | **RPE Slider** | Discrete slider-based input for RPE (1-10 in 0.5 steps). | RPE-[01-03] | 1 |
| 6 | **Jump Controls** | Buttons for rapid adjustment (±1, ±5, ±10, ±25). | STEP-[01-02] | 1 |
| 7 | **UI Integration** | Replace current inputs and sync with global state. | INT-[01-03] | 2 |

---

### Phase Details

#### Phase 4: Swipeable Tape Measure
**Goal:** Implementation of the tape measure component using SVG and pointer events.
**Requirements:**
- TAPE-01: Swipe to adjust weight
- TAPE-02: Swipe to adjust reps
- TAPE-03: Snap to increments (0.5/1.0)
- TAPE-04: Desktop support (click to jump)
- TAPE-05: Scroll locking (`touch-action: none`)
**Success Criteria:**
1. Component can be swiped on mobile and clicked on desktop.
2. Value updates correctly based on swipe distance and increments.

#### Phase 5: RPE Slider
**Goal:** Implementation of the discrete slider for RPE.
**Requirements:**
- RPE-01: Adjust RPE via slider (1-10)
- RPE-02: Snap to 0.5 increments
- RPE-03: Prominent value display
**Success Criteria:**
1. Slider operates smoothly and snaps accurately.
2. Value display updates instantly.

#### Phase 6: Jump & Step Controls
**Goal:** Implementation of "Big Step" and "Small Step" buttons.
**Requirements:**
- STEP-01: Big Step buttons (±5, ±10, ±25)
- STEP-02: Small Step buttons (±1)
**Success Criteria:**
1. Buttons correctly modify the associated tape measure value.

#### Phase 7: UI Integration & Refinement
**Goal:** Integration into the main app and mobile-first refinement.
**Requirements:**
- INT-01: Replace existing number inputs
- INT-02: Sized for thumb interaction
- INT-03: Synchronize with global state (`WorkoutState`)
**Success Criteria:**
1. Recording a set in the app works end-to-end without opening the keyboard.
2. Components are usable and well-spaced on a mobile device.

---
*Roadmap defined: 2026-02-27*
