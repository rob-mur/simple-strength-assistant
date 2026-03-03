Feature: Streamlined Workout Flow
  In order to start my workouts quickly
  As a user
  I want to initiate a workout session directly from the Library tab

  Scenario: Start a session from the Library
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then the application should switch to the "Workout" tab
    And a new session for "Bench Press" should be active

  Scenario: Workout tab shows empty state when no session is active
    Given no workout session is currently active
    When I open the "Workout" tab
    Then I should see a message saying "No active session"
    And I should see a button that says "Go to Library"
