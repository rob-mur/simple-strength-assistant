Feature: Tab bar stays fixed during scrolling
  As a user with a lot of workout data
  I want the bottom navigation bar to always be visible
  So I can switch tabs without scrolling to the bottom

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Tab bar is visible when page content exceeds viewport height
    Given the viewport is set to a small height of 400px
    When I am on the workout page
    Then the tab bar should be visible within the viewport

  Scenario: Tab bar remains visible after scrolling through long content
    Given the viewport is set to a small height of 400px
    And there is enough content to scroll
    When I scroll to the bottom of the content area
    Then the tab bar should be visible within the viewport

  Scenario: Page content area is independently scrollable
    Given the viewport is set to a small height of 400px
    And there is enough content to scroll
    Then the content area should be scrollable
    And the tab bar should be visible within the viewport

  Scenario: Short content pages show no layout regression
    When I am on the workout page
    Then the tab bar should be visible within the viewport
    And there should be no extra gap below the tab bar
