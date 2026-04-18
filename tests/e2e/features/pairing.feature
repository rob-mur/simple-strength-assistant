@e2e
Feature: Device Pairing Flow
  As a user
  I want to set up sync via the pairing flow
  So that my devices can sync workout data

  Background:
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds

  Scenario: Sync indicator updates after setting up sync
    When I navigate to the Settings tab
    And I click the setup sync button
    Then I should see the QR code display
    When I dismiss the QR code display
    Then the sync status indicator should not show the idle state

  Scenario: Sync section shows paired status after setup
    When I navigate to the Settings tab
    And I click the setup sync button
    When I dismiss the QR code display
    Then I should see the paired sync status
