Feature: Shell layout — fixed navigation bar
  As a user
  I want the bottom navigation bar to always be visible
  So that I can navigate at any time regardless of how long the page content is

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Tab bar is visible at the bottom of the screen on long-content pages
    Given I am on the workout history page with multiple entries
    Then the tab bar should be visible within the viewport
    And the tab bar should not be scrolled off the bottom of the screen

  Scenario: Page content scrolls independently above the fixed tab bar
    Given I am on the workout history page with multiple entries
    When I scroll to the bottom of the content area
    Then the tab bar should be visible within the viewport
    And the tab bar should not have moved vertically

  Scenario: Short content pages show the tab bar correctly
    Then the tab bar should be visible within the viewport
    And the page content area should be scrollable

  Scenario: Tab bar remains fixed after switching tabs
    Given I am on the workout history page with multiple entries
    When I click on the "Library" tab
    Then the tab bar should be visible within the viewport
    When I click on the "Workout" tab
    Then the tab bar should be visible within the viewport
