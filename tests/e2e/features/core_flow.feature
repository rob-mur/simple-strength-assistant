@e2e
Feature: Core user journey
  As a user
  I want to open the app, create a database, log a set, and view it in history
  So that I know the full end-to-end flow works against the deployed environment

  Scenario: Open app, create DB, log a set, view in history, see sync indicator
    Given I have a fresh context and clear storage
    And I create a new database
    And I start a test session with "Bench Press"
    When I log a set in the current session
    Then the in-progress sets section should show "Today's Sets (1 set)"
    When I click the history icon in the session header
    Then I should be on the history page
    And the history feed should contain at least 1 set row
    When I click the back button on the history page
    And I click on the "Workout" tab
    Then the sync status indicator should be visible
