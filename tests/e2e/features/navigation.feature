Feature: App Navigation
  As a user
  I want the app to remember my position in each tab
  So I can switch between tabs without losing my place

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Restore previous location on tab switch
    When I navigate to the workout history
    Then I should be on the history page
    When I click on the "Library" tab
    Then I should be on the library page
    When I click on the "Workout" tab
    Then I should be on the history page

  Scenario: Tapping active tab returns to root
    When I navigate to the workout history
    Then I should be on the history page
    When I click on the "Workout" tab
    Then I should be on the workout root page

  Scenario: Browser back button navigates back
    When I navigate to the workout history
    Then I should be on the history page
    When I press the browser back button
    Then I should be on the workout root page
