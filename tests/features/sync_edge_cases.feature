Feature: Sync edge cases — migration integrity and JS-Rust boundary
  In order to trust that sync upgrades and cross-language boundaries work
  As a developer
  I want automated checks for migration safety and signal propagation

  # ── Migration integrity ──────────────────────────────────────────────

  # QA: existing data survives crsql_as_crr migration
  Scenario: Existing exercises survive CRR migration
    Given a database with exercises before CRR migration
    When the crsql_as_crr migration runs
    Then all pre-existing exercises should still be present

  # QA: app functional after migration without re-pairing
  Scenario: App is functional after CRR migration without re-pairing
    Given a database with exercises and sync credentials before CRR migration
    When the crsql_as_crr migration runs
    Then the sync credentials should still be present
    And the app should be in a ready state

  # ── JS ↔ Rust boundary ──────────────────────────────────────────────

  # QA: onConnected / onDisconnected drive sync status signal
  Scenario: Connected and disconnected events update sync status
    Given the sync status is "idle"
    When the JS bridge reports connected
    Then the sync status should be "syncing"
    When the JS bridge reports sync complete
    Then the sync status should be "up to date"
    When the JS bridge reports disconnected
    Then the sync status should be "error"

  # QA: onSyncError surfaces error state in UI
  Scenario: Sync error from JS bridge surfaces error state
    Given the sync status is "up to date"
    When the JS bridge reports a sync error "WebSocket connection refused"
    Then the sync status should be "error"
    And the sync error message should contain "WebSocket connection refused"
