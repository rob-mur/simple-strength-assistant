Feature: Exercise Detail View
  As a user
  I want to see the details and history of a specific exercise
  So that I can review my progress and start a session

  @fast
  Scenario: Open detail view from library and start a session
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Bench Press" card
    Then the URL should contain "/library/"
    And the user should see "BENCH PRESS" in the header
    And the user should see the history feed for "Bench Press"
    When the user taps the Start button in the detail view
    Then the user should be on the Workout tab
    And a session for "Bench Press" should be active

  @fast
  Scenario: Edit exercise from detail view
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Squat" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Squat" card
    And the user taps the Edit button in the detail view
    Then the user should see the "Edit Exercise" form
    When the user changes the exercise name to "Heavy Squat"
    And the user saves the exercise
    Then the user should see "HEAVY SQUAT" in the header

  @fast
  Scenario: START creates ad-hoc plan with active session
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Bench Press" card
    And the user taps the Start button in the detail view
    Then the user should be on the Workout tab
    And a session for "Bench Press" should be active
    And an active plan should exist with the exercise

  @fast
  Scenario: Back button returns to library
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Deadlift" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Deadlift" card
    And the user taps the back button in the detail view
    Then the user should be on the Library tab

  @fast
  Scenario: Library card has no inline edit control
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    Then the library card for "Bench Press" has no edit button

  @fast
  Scenario: Library card displays a chevron navigation affordance
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    Then the library card for "Bench Press" displays a chevron
