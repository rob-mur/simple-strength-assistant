Feature: Exercise Search and Filtering
  As a user
  I want to search and filter my exercise list
  So that I can quickly find the exercise I want to perform

  @e2e
  Scenario: End-to-end user flow for searching exercises
    Given I have a fresh context and clear storage
    And I create a new database
    And the user is on the Library tab with multiple exercises
    When the user searches for a specific exercise
    Then the list should instantly filter to show only matching exercises
    When the user clears the search
    Then the list should show all exercises again