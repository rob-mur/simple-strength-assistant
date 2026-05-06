@fast
Feature: Block archive when exercise is the current session
  As a user recording a workout
  I want the archive action to be blocked for the exercise I am currently recording
  So that I cannot accidentally archive it mid-set

  @mobile
  Scenario: Trash icon is disabled on the active exercise detail screen during a workout
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the database contains "Squat" as a weighted exercise
    And I start a test session with "Bench Press"
    When the user navigates to the Library tab
    And the user taps on the "Bench Press" card
    Then the archive button is disabled for the current session exercise

  @mobile
  Scenario: Tapping the disabled trash icon does not open the archive dialog
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And I start a test session with "Bench Press"
    When the user navigates to the Library tab
    And the user taps on the "Bench Press" card
    And the user attempts to tap the disabled archive button
    Then the archive dialog is not shown

  @mobile
  Scenario: Trash icon is enabled on a different exercise during an active workout
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the database contains "Squat" as a weighted exercise
    And I start a test session with "Bench Press"
    When the user navigates to the Library tab
    And the user taps on the "Squat" card
    Then the archive button is enabled for the non-session exercise

  @mobile
  Scenario: Trash icon is enabled when no workout session is active
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains "Bench Press" as a weighted exercise
    And the user is on the Library tab
    When the user taps on the "Bench Press" card
    Then the archive button is enabled for the non-session exercise
