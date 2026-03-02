Feature: Exercise Management in Library
  In order to manage my workout data efficiently
  As a user
  I want to add and edit exercises directly from the Library tab

  Scenario: Add a new weighted exercise
    Given the Library tab is open
    When I click the "Add Exercise" button
    And I enter "Overhead Press" as the exercise name
    And I set the exercise type to "Weighted"
    And I set the minimum weight to "20" kg
    And I set the weight increment to "2.5" kg
    And I save the new exercise
    Then "Overhead Press" should appear in the exercise list
    And it should display a minimum weight of "20kg" and increment of "2.5kg"

  Scenario: Edit an existing exercise
    Given an exercise named "Squat" exists in the library
    And the Library tab is open
    When I select the "Squat" exercise
    And I click the "Edit" button
    And I change the minimum weight to "60" kg
    And I save the changes
    Then the "Squat" exercise should show a minimum weight of "60kg"
