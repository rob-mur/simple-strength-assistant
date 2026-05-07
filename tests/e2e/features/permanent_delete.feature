@fast
Feature: Permanent delete exercise with full cascade
  As a user
  I want to permanently delete an exercise and all its history
  So that it is completely removed from the app

  @mobile
  Scenario: Escalation path — archive dialog → permanent delete → exercise gone (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Bench Press" card
    And the user taps the archive button on the detail view
    Then the archive dialog is shown for "Bench Press"
    And the archive dialog has a "Delete permanently →" link
    When the user taps the "Delete permanently →" link
    Then the permanent-delete dialog is shown for "Bench Press"
    And the permanent-delete dialog has a "Delete forever" button
    When the user taps "Delete forever"
    Then the user is on the Library tab
    And "Bench Press" is not in the active exercise list
    When the user turns on the show archived toggle
    Then "Bench Press" is not in the archived exercise list

  @mobile
  Scenario: Direct path — archived exercise trash → permanent delete → gone (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Pull-ups" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Pull-ups" card
    And the user taps the archive button on the detail view
    And the user confirms the archive dialog
    Then the user is on the Library tab
    When the user turns on the show archived toggle
    And the user taps on the "Pull-ups" card
    Then the user sees the Unarchive button instead of START
    When the user taps the trash icon on the archived detail view
    Then the permanent-delete dialog is shown for "Pull-ups"
    And the archive dialog is not shown
    When the user taps "Delete forever"
    Then the user is on the Library tab
    And "Pull-ups" is not in the active exercise list
    When the user turns on the show archived toggle
    Then "Pull-ups" is not in the archived exercise list
