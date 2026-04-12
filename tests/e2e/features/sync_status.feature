@fast
Feature: Sync Status Indicator
  As a user
  I want to see the current sync state at a glance
  So that I know whether my data is synced, syncing, or in error

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  # QA: When no sync is configured, the indicator shows idle/no sync configured state
  Scenario: Indicator shows idle state when no sync is configured
    Then the sync status indicator should be visible
    And the sync status indicator should show the idle state

  # QA: Indicator is visible without any user action
  Scenario: Indicator is visible without user interaction
    Then the sync status indicator should be visible

  # QA: Indicator does not overlap or obscure main workout UI
  Scenario: Indicator does not obscure main workout UI
    Then the sync status indicator should be visible
    And the main workout interface should not be obscured by the sync indicator
