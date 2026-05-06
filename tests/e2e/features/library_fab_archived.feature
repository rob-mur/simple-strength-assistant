@fast
Feature: Library header FAB and Show Archived toggle
  As a user
  I want a FAB to add exercises and a toggle to view archived exercises
  So that the library header is clean and archived exercises are accessible

  @mobile
  Scenario: FAB is visible on the Library list view (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the user is on the Library tab
    Then the add exercise FAB is visible
    And the show archived toggle is visible

  @mobile
  Scenario: FAB is hidden on the Library exercise detail view (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains standard exercises
    And the user is on the Library tab
    When the user opens an exercise detail
    Then the add exercise FAB is hidden

  @mobile
  Scenario: Show archived toggle switches list state (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains standard exercises
    And the user is on the Library tab
    When the user turns on the show archived toggle
    Then the empty archived state message is shown

  @mobile
  Scenario: Search works within archived list (mobile)
    Given I have a fresh context and clear storage
    And I create a new database
    And the database contains standard exercises
    And the user is on the Library tab
    When the user turns on the show archived toggle
    And the user searches for "bench"
    Then the empty archived state message is shown
