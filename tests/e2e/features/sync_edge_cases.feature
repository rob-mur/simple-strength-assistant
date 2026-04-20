@e2e @sync-backend
Feature: Sync edge cases for cr-sqlite
  As a user syncing across devices
  I want sync to handle offline, reconnection, and isolation correctly
  So that my data is never lost or leaked between sync pairs

  # ── Offline / reconnect ────────────────────────────────────────────────

  # QA: device changes while offline → reconnects → changes appear on paired device
  Scenario: Offline edits sync after reconnect
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I set up sync and copy the sync code
    And I wait for sync to complete
    And the device goes offline
    And I add an exercise called "Offline Exercise"
    And the device goes back online
    And I wait for sync to complete
    And I clear storage and reload as a new device
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I join sync with the copied sync code
    And I wait for sync to complete
    Then I should see the exercise "Offline Exercise" in the library

  # QA: device loses connection mid-sync → reconnects → sync completes
  Scenario: Interrupted sync recovers after reconnect
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I add an exercise called "Mid-Sync Exercise"
    And I set up sync and copy the sync code
    And the device goes offline briefly during sync
    And the device goes back online
    And I wait for sync to complete
    Then the sync should complete without errors

  # ── Concurrent offline writes (auto-merge) ─────────────────────────────

  # QA: both devices change while offline → reconnect → all changes visible, no conflict screen
  # fixme: requires WebSocket sync server endpoint (/ws) — blocked until server is updated
  @fixme
  Scenario: Concurrent offline edits merge without conflict
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I add an exercise called "Device A Offline Ex"
    And I set up sync and copy the sync code
    And I wait for sync to complete
    # Set up Device B
    And I clear storage and reload as a new device
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I join sync with the copied sync code
    And I wait for sync to complete
    # Both go offline and make independent edits
    And the device goes offline
    And I add an exercise called "Device B Solo Ex"
    # Reconnect — CRR auto-merge should surface both
    And the device goes back online
    And I wait for sync to complete
    Then I should see the exercise "Device A Offline Ex" in the library
    And I should see the exercise "Device B Solo Ex" in the library
    And no conflict resolution screen should be visible

  # ── Room isolation ─────────────────────────────────────────────────────

  # QA: two independent pairs can't see each other's data
  # fixme: requires WebSocket sync server endpoint (/ws) — blocked until server is updated
  @fixme
  Scenario: Separate sync rooms are isolated
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I add an exercise called "Room A Exercise"
    And I set up sync and copy the sync code
    And I wait for sync to complete
    # Create a completely independent second pair
    And I clear storage and reload as a new device
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    When I add an exercise called "Room B Exercise"
    And I set up sync and copy the sync code as second room
    And I wait for sync to complete
    Then I should see the exercise "Room B Exercise" in the library
    And I should not see the exercise "Room A Exercise" in the library
