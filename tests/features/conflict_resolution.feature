Feature: Conflict Resolution UI

  Background:
    Given the app is in the Ready state

  Scenario: Conflict screen appears when sync client reports unresolved conflicts
    When the sync client reports 2 unresolved conflicts
    Then the conflict resolution screen is displayed
    And the screen shows 2 conflict cards

  Scenario: Each conflicting record shows both versions with enough context
    When the sync client reports a conflict for exercise "Bench Press" vs "Flat Bench Press"
    Then the conflict resolution screen is displayed
    And the conflict card shows "Device A (Local)" with field "name" value "Bench Press"
    And the conflict card shows "Device B (Remote)" with field "name" value "Flat Bench Press"
    And the differing field "name" is visually highlighted

  Scenario: User can select one version per record
    When the sync client reports a conflict for exercise "Bench Press" vs "Flat Bench Press"
    And the user selects version A for the conflict
    Then version A is marked as selected
    And version B is not marked as selected

  Scenario: Selecting one version does not auto-select others
    When the sync client reports 2 unresolved conflicts
    And the user selects version A for the first conflict
    Then only the first conflict has a selection
    And the resolve button is disabled

  Scenario: Resolve button is enabled only when all conflicts are resolved
    When the sync client reports 2 unresolved conflicts
    And the user selects version A for the first conflict
    And the user selects version B for the second conflict
    Then the resolve button is enabled

  Scenario: Conflict screen is never shown when there are no conflicts
    When there are no pending conflicts
    Then the conflict resolution screen is not displayed
    And the normal app content is shown
