Feature: Full workout history view
  As a user
  I want to view my complete workout history
  So I can review my progress across exercises and days

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  # AC #8: "View workout history" button on idle Workout tab
  Scenario: Idle Workout tab has "View workout history" button that navigates to all-exercises view
    Then I should see the "View workout history" button on the idle Workout tab
    When I click the "View workout history" button
    Then I should be on the history page
    And the "All Exercises" toggle should be active

  # AC #1: /workout/history renders all-exercises feed by default
  Scenario: /workout/history defaults to All Exercises scope
    When I navigate directly to the history page
    Then I should be on the history page
    And the "All Exercises" toggle should be active

  # AC #2: /workout/history/:exercise_id defaults to the exercise tab
  Scenario: /workout/history/:exercise_id defaults to per-exercise scope
    Given I start a test session with "Squat"
    And I log a set in the current session
    When I click the history icon in the session header
    Then I should be on the history page
    And the exercise toggle should be active

  # AC #7: History icon in active session header
  Scenario: History icon is visible in active session header
    Given I start a test session with "Squat"
    Then the history icon should be visible in the session header

  # AC #3: Toggle switches between per-exercise and all-exercises feeds
  Scenario: Toggle switches between exercise and all-exercises feeds
    Given I start a test session with "Squat"
    And I log a set in the current session
    And I finish any active session
    And I start a test session with "Bench Press"
    And I log a set in the current session
    And I finish any active session
    When I navigate directly to the history page
    Then the "All Exercises" toggle should be active
    And I should see "Squat" in the history feed
    And I should see "Bench Press" in the history feed

  # AC #4 + AC #5: Reverse-chronological day groups, multiple exercises share one header
  Scenario: Feed shows correct day groups with multiple exercises sharing one date header
    Given I start a test session with "Deadlift"
    And I log a set in the current session
    And I finish any active session
    And I start a test session with "Overhead Press"
    And I log a set in the current session
    And I finish any active session
    When I navigate directly to the history page
    Then the history feed should have exactly 1 day group
    And the day group should contain 2 exercise sub-groups

  # AC #9: Sets logged during active session appear in history feed immediately
  Scenario: Sets logged during an active session appear in the history feed
    Given I start a test session with "Deadlift"
    When I log a set in the current session
    And I click the history icon in the session header
    Then I should be on the history page
    And the history feed should contain at least 1 set row

  # AC #6: Infinite scroll loads additional pages
  Scenario: Infinite scroll loads the next page when more than 20 sets exist
    Given I start a test session with "Deadlift"
    And I have logged 25 sets for "Deadlift" in a previous session
    When I navigate directly to the history page
    Then the history feed should initially show 20 set rows
    When I scroll to the bottom of the history feed
    Then the history feed should show more than 20 set rows

  # AC #10 (combined): Correct default scope from each entry point
  Scenario: Accessing history from idle tab defaults to All Exercises
    When I click the "View workout history" button
    Then the "All Exercises" toggle should be active

  Scenario: Accessing history from active session defaults to current exercise
    Given I start a test session with "Squat"
    And I log a set in the current session
    When I click the history icon in the session header
    Then the exercise toggle should be active

  # AC #11: Exercise filter dropdown on idle history view
  Scenario: Idle history view has exercise filter dropdown that can filter by exercise
    Given I start a test session with "Squat"
    And I log a set in the current session
    And I finish any active session
    And I start a test session with "Bench Press"
    And I log a set in the current session
    And I finish any active session
    When I click the "View workout history" button
    Then the exercise filter selector should be visible
    And I should see "Squat" in the history feed
    And I should see "Bench Press" in the history feed
    When I select "Squat" from the exercise filter
    Then the history feed should show only "Squat" sets
