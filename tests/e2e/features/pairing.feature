@e2e
Feature: Device Pairing Flow
  As a user
  I want to see sync status and pairing controls
  So that I can manage device synchronisation

  Background:
    Given I open the app with real sync backend and clear storage
    When I click "Create New Database"
    Then the app should reach the workout view within 10 seconds

  Scenario: Settings shows paired status with auto-generated credentials
    When I navigate to the Settings tab
    Then I should see the paired sync status

  Scenario: Pair another device shows QR code
    When I navigate to the Settings tab
    And I click the pair another device button
    Then I should see the QR code display
