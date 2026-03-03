Feature: Exercise List Display
  As a user
  I want to see a list of my exercises
  So that I can browse and select exercises for my workout

  @e2e
  Scenario: End-to-end user flow for viewing the exercise list
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains standard exercises
    And the user is on the Library tab
    Then the user should see a list of exercises
    And each exercise should display its name and type badge