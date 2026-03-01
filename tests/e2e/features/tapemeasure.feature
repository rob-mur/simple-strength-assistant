Feature: TapeMeasure Component E2E

  Background:
    Given I have a fresh context and clear storage
    And I create a new database
    And I finish any active session
    And I start a test session with "Test Bench Press"

  Scenario: Swipe drag gesture updates value
    When I swipe the reps TapeMeasure left to increase value
    Then the reps TapeMeasure value should increase

  Scenario: Click on tick mark jumps to value
    When I click on a different tick mark in the reps TapeMeasure
    Then the reps TapeMeasure value should jump to the clicked value

  Scenario: Ghost click prevention after drag
    When I drag the TapeMeasure and immediately click
    Then the TapeMeasure value should not change due to click suppression

  Scenario: SVG rendering and transform updates
    When I drag the TapeMeasure
    Then the SVG transform should change

  Scenario: Edge clamping prevents overflow
    When I drag the TapeMeasure far beyond maximum
    Then the TapeMeasure should not crash and remain visible
