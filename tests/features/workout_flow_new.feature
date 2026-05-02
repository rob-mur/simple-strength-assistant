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

  # AC #8: "View workout history" button on idle Workout tab
  Scenario: Idle Workout tab shows a View workout history button
    Given no workout session is currently active
    When I open the "Workout" tab
    Then I should see a button that says "View workout history"

  # AC #7: History icon appears in active session header
  Scenario: Active session header shows history navigation icon
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then I should see a message saying "View exercise history"

  # Issue 74: Finish Workout Session button must be removed
  Scenario: Active workout page does not show Finish Workout Session button
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then I should not see a button that says "Finish Workout Session"

  # Issue 74: Switching exercise starts a fresh session with zero completed sets
  Scenario: Switching to a new exercise starts fresh with zero completed sets
    Given an active session for "Bench Press" with completed sets
    When I switch to exercise "Squat"
    Then the new session for "Squat" should have zero completed sets

  # Issue 154: Exercise name appears only in the tab strip, not duplicated below
  Scenario: Active workout view does not duplicate the exercise name below the tab strip
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then I should not see a duplicate exercise header card

  # Issue 154: History icon remains accessible after removing the duplicate header
  Scenario: History icon is accessible from the active workout input area
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then I should see a history icon in the input area

  # Issue 152: End Workout clears session so planning screen shows
  Scenario: End Workout returns to the planning screen
    Given an active session for "Bench Press" with completed sets
    When the workout plan is ended
    Then no workout session should be active
    And no workout plan should be active
