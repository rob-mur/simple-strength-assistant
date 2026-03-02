Feature: Exercise Search and Filtering
  As a user
  I want to search and filter my exercise list
  So that I can quickly find the exercise I want to perform

  @unit
  Scenario: Instant text filtering
    Given I am on the Library tab
    And the following exercises exist:
      | Name           | Type       |
      | Back Squat     | Weighted   |
      | Front Squat    | Weighted   |
      | Push-up        | Bodyweight |
      | Pull-up        | Bodyweight |
    When I type "squat" into the search bar
    Then I should see "Back Squat" in the exercise list
    And I should see "Front Squat" in the exercise list
    And I should not see "Push-up" in the exercise list
    And I should not see "Pull-up" in the exercise list

  @unit
  Scenario: Case-insensitive search
    Given I am on the Library tab
    And the following exercises exist:
      | Name           | Type       |
      | Back Squat     | Weighted   |
    When I type "SQUAT" into the search bar
    Then I should see "Back Squat" in the exercise list

  @unit
  Scenario: No matching exercises
    Given I am on the Library tab
    And the following exercises exist:
      | Name           | Type       |
      | Back Squat     | Weighted   |
    When I type "Deadlift" into the search bar
    Then I should see the "No matching exercises" empty state message
    And the exercise list should be empty

  @e2e
  Scenario: End-to-end user flow for searching exercises
    Given the user is on the Library tab with multiple exercises
    When the user searches for a specific exercise
    Then the list should instantly filter to show only matching exercises
    When the user clears the search
    Then the list should show all exercises again
