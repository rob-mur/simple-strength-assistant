@e2e @sync-backend
Feature: Sync integration against real backend
  As a developer debugging sync errors
  I want to verify the app can sync against the real sync server
  So I can diagnose issues seen in the deployed environment

  Scenario: App completes a sync cycle against the real sync backend
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    And the sync should complete without errors

  Scenario: Sync push returns a successful HTTP response
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    And no sync network errors should appear in the console
