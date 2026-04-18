@fast
Feature: Device Pairing Flow
  As a user
  I want to set up sync via the pairing flow
  So that my devices can sync workout data

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Sync indicator updates after setting up sync
    Then the sync status indicator should show the idle state
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
