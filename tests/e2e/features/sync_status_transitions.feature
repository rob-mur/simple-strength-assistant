@e2e @sync-backend
Feature: Sync status indicator transitions
  As a user
  I want the sync status indicator to reflect connection changes in real time
  So that I know when my data is synced, offline, or syncing

  # QA: synced → offline → syncing → synced transition chain
  Scenario: Status indicator transitions through synced-offline-syncing-synced
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I set up sync and copy the sync code
    And I wait for sync to complete
    Then the sync status indicator should show a synced state
    When the device goes offline
    Then the sync status indicator should show an offline or error state
    When the device goes back online
    Then the sync status indicator should transition back to synced
