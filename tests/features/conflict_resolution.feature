Feature: Conflict Resolution UI
  In order to resolve data conflicts between devices
  As a user
  I want to see and resolve conflicts before continuing to use the app

  # QA: conflict screen is shown when sync client reports unresolved conflicts
  Scenario: Conflict resolution screen is shown when conflicts are present
    Given the sync client has reported 1 unresolved conflict
    When I view the app
    Then the conflict resolution screen should be visible
    And the main workout UI should not be visible

  # QA: each conflicting record shows both versions with enough context
  Scenario: Each conflict shows both versions with labels
    Given the sync client has reported a conflict for record "exercise-uuid-1" with versions "Bench Press" and "Bench Presss"
    When I view the conflict resolution screen
    Then I should see version A labelled "Device A"
    And I should see version B labelled "Device B"
    And I should see the value "Bench Press" for version A
    And I should see the value "Bench Presss" for version B

  # QA: user can select exactly one version per conflicting record
  Scenario: User can select one version per conflicting record
    Given the sync client has reported 1 unresolved conflict
    When I view the conflict resolution screen
    Then I should see selectable options for version A and version B
    And selecting one version should not auto-select any other record's version

  # QA: resolve button is only available after all conflicts are resolved
  Scenario: Resolve button appears only after all conflicts have a selection
    Given the sync client has reported 2 unresolved conflicts
    When I view the conflict resolution screen
    Then the resolve button should be disabled or absent
    When I select version A for all conflicts
    Then the resolve button should be available

  # QA: if sync completes with zero conflicts, the conflict resolution screen is never shown
  Scenario: Conflict resolution screen is not shown when there are no conflicts
    Given the sync client has reported 0 unresolved conflicts
    When I view the app
    Then the conflict resolution screen should not be visible

  # QA: rejected version does not appear after resolution (resolved state transitions away)
  Scenario: After resolving all conflicts the resolution screen is dismissed
    Given the sync client has reported 1 unresolved conflict
    And I have selected version A for all conflicts
    When I confirm the resolution
    Then the conflict resolution screen should not be visible
