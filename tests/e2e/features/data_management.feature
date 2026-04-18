@fast
Feature: Data import
  As a user
  I want to import my workout database
  So I can restore my data on any browser

  Background:
    Given I have a fresh context and clear storage
    And I create a new database
    And I navigate to the Settings tab

  Scenario: Import button is visible on the Settings tab
    Then I should see the import button

  Scenario: Importing a non-SQLite file shows a graceful error
    When I import an invalid file
    Then I should see an import error message
