@e2e
Feature: Device Pairing Flow
  As a user
  I want to set up sync and pair devices
  So that my workout data syncs across devices

  Background:
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds

  Scenario: Sync indicator updates after setting up sync
    When I navigate to the Settings tab
    And I click the setup sync button
    Then I should see the sync code display
    And I should see the copy sync code button
    When I dismiss the sync code display
    Then the sync status indicator should not show idle after sync completes

  Scenario: Settings shows paired status after setup
    When I navigate to the Settings tab
    And I click the setup sync button
    When I dismiss the sync code display
    Then I should see the paired sync status
