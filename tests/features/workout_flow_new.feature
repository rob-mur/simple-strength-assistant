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

  # Issue 162: Starting a plan auto-starts a session on the first exercise
  Scenario: Starting a plan auto-starts a session on the first exercise
    Given a plan with exercises "Squat", "Bench Press", "Deadlift"
    When the plan is started
    Then the active session should be for "Squat"

  # Issue 164: Over-plan banner is removed
  Scenario: Over-plan warning banner is absent during active session
    Given an active session for "Bench Press" with completed sets
    Then the over-plan warning banner should not be present

  # Issue 164: Set-count badge renders in warning colour when completed > planned
  Scenario: Set-count badge uses warning colour when completed sets exceed planned
    Given an exercise tab with 3 completed sets and 2 planned sets
    Then the set-count badge should render in warning colour

  # Issue 164: Set-count badge uses default colour when completed <= planned
  Scenario: Set-count badge uses default colour when completed sets are within plan
    Given an exercise tab with 2 completed sets and 3 planned sets
    Then the set-count badge should render in default colour

  # Issue 167: Complete Workout via three-dot menu
  Scenario: Complete Workout via menu sets ended_at and clears plan
    Given an active session for "Bench Press" with completed sets
    When the user selects Complete Workout from the menu and confirms
    Then no workout session should be active
    And no workout plan should be active
    And the plan should have ended_at set

  # Issue 167: Top-right End Workout button is removed
  Scenario: Active workout view does not show End Workout button
    Given an active session for "Bench Press" with completed sets
    Then the End Workout button should not be present

  # Issue 167: Cancel on confirmation dialog leaves plan unchanged
  Scenario: Cancelling the Complete Workout confirmation leaves plan active
    Given an active session for "Bench Press" with completed sets
    When the user selects Complete Workout from the menu and cancels
    Then a workout session should still be active
    And a workout plan should still be active
