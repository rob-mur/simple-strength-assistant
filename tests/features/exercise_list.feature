Feature: Exercise List Display
  As a user
  I want to see a list of my exercises
  So that I can browse and select exercises for my workout

  @unit
  Scenario: Empty exercise list
    Given the app is loaded
    And there are no exercises in the database
    When I navigate to the Library tab
    Then I should see the "No exercises yet" empty state message

  @unit
  Scenario: Viewing populated exercise list
    Given the app is loaded
    And the following exercises exist:
      | Name      | Type       |
      | Squat     | Weighted   |
      | Push-up   | Bodyweight |
    When I navigate to the Library tab
    Then I should see "Squat" in the exercise list
    And I should see "Push-up" in the exercise list

  @unit
  Scenario: Verifying exercise type badges
    Given the app is loaded
    And the following exercises exist:
      | Name      | Type       |
      | Squat     | Weighted   |
      | Push-up   | Bodyweight |
    When I navigate to the Library tab
    Then the "Squat" exercise should have a "Weighted" badge
    And the "Push-up" exercise should have a "Bodyweight" badge

  @e2e
  Scenario: End-to-end user flow for viewing the exercise list
    Given the user is on the Library tab
    And the database contains standard exercises
    Then the user should see a list of exercises
    And each exercise should display its name and type badge
