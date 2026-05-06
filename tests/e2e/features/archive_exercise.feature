@fast
Feature: Archive and unarchive an exercise (no plan cascade)
  As a user
  I want to archive exercises I no longer use
  So that they are hidden from the library but recoverable

  @mobile
  Scenario: Archive a basic exercise and verify it moves to the archived list (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Bench Press" card
    And the user taps the archive button on the detail view
    Then the archive dialog is shown for "Bench Press"
    And the archive dialog shows "0 future plans will be deleted"
    When the user confirms the archive dialog
    Then the user is on the Library tab
    And "Bench Press" is not in the active exercise list
    When the user turns on the show archived toggle
    Then "Bench Press" appears in the archived list with an ARCHIVED badge

  @mobile
  Scenario: Unarchive an exercise and verify it returns to the active list (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Bench Press" card
    And the user taps the archive button on the detail view
    And the user confirms the archive dialog
    Then the user is on the Library tab
    When the user turns on the show archived toggle
    And the user taps on the "Bench Press" card
    Then the user sees the Unarchive button instead of START
    When the user taps the Unarchive button
    Then the user is on the Library tab
    And "Bench Press" is in the active exercise list
