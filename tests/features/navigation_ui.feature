Feature: Navigation Bar UI
  In order to have a professional and usable mobile experience
  As a user
  I want the bottom navigation bar to be correctly positioned and handle safe areas

  @mobile
  Scenario: Navigation bar is fixed to the bottom
    Given the application is open
    Then the bottom navigation bar should be visible
    And the bottom navigation bar should have bottom padding for safe areas
    And the navigation bar should remain at the bottom when scrolling
