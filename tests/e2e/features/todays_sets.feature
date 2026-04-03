Feature: Today's Sets section label in active session
  As a user logging a workout
  I want the in-progress sets section to be clearly labelled "Today's Sets"
  So I can distinguish it from my historical previous sessions data

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  # Issue #73: Rename "History" section to "Today's Sets" for in-progress sessions
  Scenario: In-progress sets section is labelled "Today's Sets" after logging a set
    Given I start a test session with "Squat"
    When I log a set in the current session
    Then the in-progress sets section should show "Today's Sets"

  Scenario: The label "History" does not appear for the current-session sets section
    Given I start a test session with "Squat"
    When I log a set in the current session
    Then the in-progress sets heading should not contain "History"

  Scenario: The set count appears correctly in the Today's Sets heading
    Given I start a test session with "Squat"
    When I log a set in the current session
    Then the in-progress sets section should show "Today's Sets (1 sets)"
