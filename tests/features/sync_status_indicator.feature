Feature: Sync Status Indicator
  In order to know whether my data is synced
  As a user
  I want a visible indicator that shows the current sync state

  # QA: visible without requiring any user action
  Scenario: Indicator is visible in the app header
    Given the app is initialized
    When I view the app
    Then I should see the sync status indicator

  # QA: idle state when no sync is configured
  Scenario: Indicator shows idle state when no sync is configured
    Given no sync is configured
    When I view the app
    Then the sync status indicator should show "No sync"
    And the sync status data attribute should be "idle"

  # QA: never synced distinguishes from error
  Scenario: Indicator shows never-synced state when app has never synced
    Given the sync status is "never synced"
    When I view the app
    Then the sync status indicator should show "Never synced"
    And the sync status data attribute should be "never-synced"

  # QA: syncing state is visually distinct
  Scenario: Indicator shows syncing state during an active sync
    Given the sync status is "syncing"
    When I view the app
    Then the sync status indicator should show "Syncing…"
    And the sync status data attribute should be "syncing"

  # QA: up to date after successful sync
  Scenario: Indicator shows up-to-date after a successful sync
    Given the sync status is "up to date"
    When I view the app
    Then the sync status indicator should show "Up to date"
    And the sync status data attribute should be "up-to-date"

  # QA: error state after failed sync
  Scenario: Indicator shows error state after a failed sync
    Given the sync status is "error"
    When I view the app
    Then the sync status indicator should show "Sync error"
    And the sync status data attribute should be "error"

  # QA: does not block the main workout UI
  Scenario: Indicator does not obstruct the workout UI
    Given the app is initialized
    When I view the app
    Then the sync status indicator should be inside the header
    And the main content area should be present
