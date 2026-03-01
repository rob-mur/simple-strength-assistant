Feature: StepControls Component E2E

  Background:
    Given I have a fresh context and clear storage
    And I create a new database
    And I finish any active session
    And I start a test session with "Test Bench Press"

  Scenario: Increment button increases value
    When I click the increment button
    Then the step control value should increase or stay at max

  Scenario: Decrement button decreases value
    When I click the decrement button
    Then the step control value should decrease or stay at min

  Scenario: Glass effect rendering on buttons
    Then the step control buttons should have the "glass" effect and shadow

  Scenario: SVG icons render correctly
    Then the step control buttons should contain valid SVG icons

  Scenario: Value clamping at boundaries
    When I click the decrement button many times to reach minimum
    And I click the decrement button again
    Then the step control value should stay at the minimum

  Scenario: Button hover and active states work
    When I hover over a step control button
    Then the button should remain visible
    When I click and hold the step control button
    Then the button should remain visible

  Scenario: Multiple step sizes are available
    Then there should be multiple increment and decrement buttons
    And the buttons should display their step values
