Feature: Bottom navigation bar stays fixed
  As a user
  I want the bottom navigation bar to always be visible
  So I can navigate between tabs regardless of how much content is on the page

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Nav bar is visible when content exceeds viewport height
    Given I have many exercises in the library
    When I click on the "Library" tab
    Then the bottom navigation bar should be visible within the viewport

  Scenario: Nav bar remains visible after scrolling long content
    Given I have many exercises in the library
    When I click on the "Library" tab
    And I scroll down the page content
    Then the bottom navigation bar should be visible within the viewport

  Scenario: Short content pages display correctly with nav bar at bottom
    When I click on the "Library" tab
    Then the bottom navigation bar should be visible within the viewport
    And the page layout should fill the viewport without excess space
