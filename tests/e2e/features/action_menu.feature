@fast
Feature: Three-dot action menu in active session
  As a user
  I want a three-dot action menu during an active workout session
  So I can access actions like View History without cluttering the main UI

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Action menu trigger is rendered next to LOG SET
    Given I start a test session with "Squat"
    Then the action menu trigger should be visible
    And the LOG SET button should be visible

  Scenario: Tapping the action menu trigger opens the bottom sheet
    Given I start a test session with "Squat"
    When I tap the action menu trigger
    Then the bottom sheet should be visible
    And the bottom sheet should contain "View History"
    And the bottom sheet should contain "Cancel"

  Scenario: Tapping View History navigates to exercise history
    Given I start a test session with "Squat"
    And I log a set in the current session
    When I tap the action menu trigger
    And I tap "View History" in the bottom sheet
    Then I should be on the history page
    And the exercise toggle should be active

  Scenario: Back from View History returns to workout
    Given I start a test session with "Squat"
    And I log a set in the current session
    When I tap the action menu trigger
    And I tap "View History" in the bottom sheet
    Then I should be on the history page
    When I click the back button on the history page
    Then I should be on the Workout tab

  Scenario: Tapping Cancel dismisses the bottom sheet
    Given I start a test session with "Squat"
    When I tap the action menu trigger
    Then the bottom sheet should be visible
    When I tap "Cancel" in the bottom sheet
    Then the bottom sheet should not be visible

  Scenario: Tapping the backdrop dismisses the bottom sheet
    Given I start a test session with "Squat"
    When I tap the action menu trigger
    Then the bottom sheet should be visible
    When I tap the bottom sheet backdrop
    Then the bottom sheet should not be visible

  Scenario: No history-icon-btn element exists in active session
    Given I start a test session with "Squat"
    Then the old history icon should not be present

  Scenario: Bottom sheet contains Complete Workout and Discard Workout
    Given I start a test session with "Squat"
    When I tap the action menu trigger
    Then the bottom sheet should be visible
    And the bottom sheet should contain "View History"
    And the bottom sheet should contain "Complete Workout"
    And the bottom sheet should contain "Discard Workout"
    And the bottom sheet should contain "Cancel"

  Scenario: Tapping Discard Workout opens confirmation dialog
    Given I start a test session with "Squat"
    When I tap the action menu trigger
    And I tap "Discard Workout" in the bottom sheet
    Then the confirmation dialog should be visible
    And the confirmation dialog title should be "Discard this workout?"

  Scenario: Cancelling discard confirmation dismisses the dialog
    Given I start a test session with "Squat"
    When I tap the action menu trigger
    And I tap "Discard Workout" in the bottom sheet
    Then the confirmation dialog should be visible
    When I tap cancel on the confirmation dialog
    Then the confirmation dialog should not be visible

  Scenario: Confirming discard returns to plan builder
    Given I start a test session with "Squat"
    And I log a set in the current session
    When I tap the action menu trigger
    And I tap "Discard Workout" in the bottom sheet
    And I confirm the discard dialog
    Then I should see the plan builder

  Scenario: Tapping Complete Workout ends the workout
    Given I start a test session with "Squat"
    And I log a set in the current session
    When I tap the action menu trigger
    And I tap "Complete Workout" in the bottom sheet
    Then I should see the plan builder
