Feature: Collapsible Previous Sessions in active workout
  As a user logging a workout
  I want to see my past sets for the current exercise
  So I can reference previous performance while training

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: History section is collapsed by default
    Given I start a test session with "Deadlift"
    Then the "Previous Sessions" section should be collapsed

  Scenario: Tapping header expands and collapses the section
    Given I start a test session with "Deadlift"
    When I tap the "Previous Sessions" header
    Then the "Previous Sessions" section should be expanded
    When I tap the "Previous Sessions" header
    Then the "Previous Sessions" section should be collapsed

  Scenario: Logged set appears in history feed immediately
    Given I start a test session with "Deadlift"
    When I log a set in the current session
    And I tap the "Previous Sessions" header
    Then the history feed should contain at least 1 set

  Scenario: Load more button loads the next page when history is long
    Given I start a test session with "Deadlift"
    And I have logged 25 sets for "Deadlift" in a previous session
    When I tap the "Previous Sessions" header
    Then the history feed should contain 20 sets
    And a "Load more" button should be visible
    When I click the "Load more" button
    Then the history feed should contain 25 sets

  Scenario: History feed updates reactively when a set is logged while the section is expanded
    Given I start a test session with "Deadlift"
    And I have logged 3 sets for "Deadlift" in a previous session
    When I tap the "Previous Sessions" header
    Then the history feed should contain 3 sets
    When I log a set in the current session
    Then the history feed should contain 4 sets

  Scenario: Expanded section shows Set, Reps, and RPE column headers
    Given I start a test session with "Deadlift"
    And I have logged 2 sets for "Deadlift" in a previous session
    When I tap the "Previous Sessions" header
    Then the history table should have "Set", "Reps", and "RPE" column headers
