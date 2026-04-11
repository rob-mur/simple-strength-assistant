Feature: OPFS persistence across page reloads
  As a returning user
  I want my data to persist when I reload the page
  So that I can trust my workout history is saved

  Scenario: Exercise data survives a full page reload (OPFS path)
    Given I have a fresh context and clear storage
    And I create a new database
    And I start a test session with "OPFS Persistence Test"
    When I reload the page
    Then I should see "OPFS Persistence Test" in the exercise library
