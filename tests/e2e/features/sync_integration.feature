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

  Scenario: Two devices sync exercises via shared sync code
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I add an exercise called "Device A Exercise"
    And I set up sync and copy the sync code
    And I wait for sync to complete
    And I clear storage and reload as a new device
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I add an exercise called "Device B Exercise"
    And I join sync with the copied sync code
    And I wait for sync to complete
    Then I should see the exercise "Device A Exercise" in the library
    And I should see the exercise "Device B Exercise" in the library
