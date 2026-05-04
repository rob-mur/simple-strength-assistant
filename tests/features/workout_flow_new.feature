Feature: Streamlined Workout Flow
  In order to start my workouts quickly
  As a user
  I want to initiate a workout session directly from the Library tab

  Scenario: Start a session from the Library
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then the application should switch to the "Workout" tab
    And the session exercise should be "Bench Press"

  Scenario: Workout tab shows plan builder when no session is active
    Given no workout session is currently active
    When I open the "Workout" tab
    Then I should see a message saying "Plan Your Workout"

  # The idle Workout tab now shows PlanBuilder with an Add Exercise button
  Scenario: Idle Workout tab shows Add Exercise button
    Given no workout session is currently active
    When I open the "Workout" tab
    Then I should see a button that says "+ Add Exercise"

  # The three-dot action menu replaces the old history icon
  Scenario: Active session header shows action menu trigger
    Given the Library tab is open
    When I select the "Bench Press" exercise
    And I click the "Start Session" button
    Then I should see a history icon in the input area

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

  # Issue 168: Discard Workout with logged sets
  Scenario: Discard Workout soft-deletes session sets and returns to PlanBuilder
    Given a started plan with exercise "Bench Press" and 2 logged sets
    When the workout is discarded
    Then no workout session should be active
    And the plan should be unstarted with exercises preserved

  # Issue 168: Discard Workout with no logged sets
  Scenario: Discard Workout with no sets un-starts the plan
    Given a started plan with exercise "Squat" and 0 logged sets
    When the workout is discarded
    Then no workout session should be active
    And the plan should be unstarted with exercises preserved

  # Issue 166: Library START creates ad-hoc one-exercise plan
  Scenario: Library START creates ad-hoc plan with active session on chosen exercise
    Given the Library tab is open
    When the user starts exercise "Bench Press" from the Library
    Then a one-exercise plan should be active with planned sets from settings
    And the active session should be for "Bench Press"
