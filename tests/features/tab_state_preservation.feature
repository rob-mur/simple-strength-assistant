Feature: Workout Session State Preservation

  @e2e
  Scenario: Active workout session persists when switching to Library and back
    Given I have an active workout session with exercise "Bench Press"
    And the current set is set 2 of 3
    When I switch to the "Library" tab
    And I switch back to the "Workout" tab
    Then I should still see exercise "Bench Press"
    And the current set should still be set 2 of 3
    And all session data should be intact

  @e2e
  Scenario: Tab selection persists after browser refresh
    Given the app is loaded
    When I switch to the "Library" tab
    And I refresh the browser
    Then the "Library" tab should still be active

  @unit
  Scenario: WorkoutState context remains accessible across tab switches
    Given a WorkoutState context with active session data
    When the tab selection changes to "Library"
    Then the WorkoutState context should remain mounted
    And the session data should remain accessible
    When the tab selection changes back to "Workout"
    Then the WorkoutState context should have the same session data

  @unit
  Scenario: localStorage correctly saves active tab selection
    Given the tab navigation component is mounted
    When I select the "Library" tab
    Then localStorage should contain key "activeTab" with value "Library"
    When I select the "Workout" tab
    Then localStorage should contain key "activeTab" with value "Workout"

  @unit
  Scenario: Tab state initialization from localStorage on mount
    Given localStorage contains key "activeTab" with value "Library"
    When the tab navigation component mounts
    Then the "Library" tab should be initially active

  @unit
  Scenario: Tab state defaults to Workout when no localStorage entry
    Given localStorage does not contain key "activeTab"
    When the tab navigation component mounts
    Then the "Workout" tab should be initially active
