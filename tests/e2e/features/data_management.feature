Feature: Data export and import
  As a user
  I want to export and import my workout database
  So I can back up my data and restore it on any browser

  Background:
    Given I have a fresh context and clear storage
    And I create a new database
    And I navigate to the Settings tab

  Scenario: Export button is visible on the Settings tab
    Then I should see the export button

  Scenario: Import button is visible on the Settings tab
    Then I should see the import button

  Scenario: Exporting the database triggers a file download
    When I click the export button
    Then a SQLite file download is triggered

  Scenario: Importing a valid SQLite file replaces current data
    Given I have exported a database with an exercise "Bench Press"
    When I import that exported file
    Then I should see "Bench Press" in the exercise library

  Scenario: Importing a non-SQLite file shows a graceful error
    When I import an invalid file
    Then I should see an import error message
