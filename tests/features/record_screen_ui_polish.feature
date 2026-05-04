Feature: Record screen UI polish (Issue 183)
  In order to keep the active workout screen uncluttered and accurate
  As a user logging sets
  I want each piece of information rendered exactly once and inputs to behave intuitively

  # Cycle 1 — counter dedup
  Scenario: Exercise tab strip shows progress dots without textual counter
    Given an exercise tab with 1 completed sets and 3 planned sets
    Then the tab strip should not contain the textual "x/N" counter
    And the tab strip should still contain progress dots

  # Cycle 2 — symmetric rep step buttons
  Scenario: Reps step buttons are symmetric
    Given an active session for "Bench Press" with completed sets
    When I render the active session
    Then the reps step-down button label should be "−5"
    And the reps step-up button label should be "+5"

  # Cycle 3 — single "Reps" word
  Scenario: Reps readout shows the word "Reps" exactly once
    Given an active session for "Bench Press" with completed sets
    When I render the active session
    Then the rendered reps readout should not contain " reps"

  # Cycle 4 — three-dot icon visibility
  Scenario: Three-dot action menu icon visibly fills its touch target
    Given an active session for "Bench Press" with completed sets
    When I render the active session
    Then the action menu trigger SVG should have a large icon class
    And the action menu trigger SVG should have filled dots without a stroke

  # Cycle 5 — slider drag survives vertical drift
  Scenario: RPE slider input declares touch-action that prevents vertical scroll cancellation
    Given an active session for "Bench Press" with completed sets
    When I render the active session
    Then the RPE slider input should declare a horizontal-only touch-action style

  # Cycle 6 — RPE value/description appear once
  Scenario: RPE numeric value appears exactly once on the recording screen
    Given an active session for "Bench Press" with completed sets
    When I render the active session
    Then the rpe-readout testid should appear exactly once
    And the rendered output should not contain a duplicate RPE value display
