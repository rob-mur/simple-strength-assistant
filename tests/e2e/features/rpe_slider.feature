Feature: RPESlider Component E2E

  Background:
    Given I have a fresh context and clear storage
    And I create a new database
    And I finish any active session
    And I start a test session with "Test Bench Press"

  Scenario: Range input interaction updates value
    When I change the RPE slider value to "8"
    Then the RPE slider value should be "8"

  Scenario: Color class changes on value update
    When I change the RPE slider value to "6"
    Then the RPE slider should have the "range-accent" class
    When I change the RPE slider value to "8"
    Then the RPE slider should have the "range-warning" class
    When I change the RPE slider value to "9.5"
    Then the RPE slider should have the "range-error" class

  Scenario: Legend text displays correct RPE description
    When I change the RPE slider value to "6"
    Then the RPE legend text should be visible

  Scenario: Keyboard navigation works
    When I focus the RPE slider
    And I press the "ArrowUp" key on the slider
    Then the RPE slider value should increase
    When I press the "ArrowDown" key on the slider
    Then the RPE slider value should decrease

  Scenario: Snapping behavior at half-point increments
    When I change the RPE slider value to "7.5"
    Then the RPE slider value should snap to a half-point increment

  Scenario: Slider bounds are enforced
    Then the RPE slider HTML attributes should be correctly set
    When I change the RPE slider value to "1"
    Then the RPE slider value should be within bounds
    When I change the RPE slider value to "10"
    Then the RPE slider value should be within bounds
