@e2e
Feature: Production boot without test mode
  As a user on the production site
  I want the app to load without crashing after creating a database
  So I can use the workout assistant

  Scenario: App reaches workout view after creating a new database
    Given I open the app without test mode and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds

  Scenario: Sync fires at most once per app load
    Given I open the app without test mode and clear storage
    When I click "Create New Database"
    Then the sync should start at most once

  Scenario: Sync indicator stays idle when sync is not configured
    Given I open the app without test mode and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds
    And the sync status indicator should show the idle state
