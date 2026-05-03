Feature: Tab Navigation UI

  @e2e
  Scenario: User can see Workout, Library, and Analysis tabs
    Given the app is loaded
    Then I should see a "Workout" tab
    And I should see a "Library" tab
    And I should see an "Analysis" tab
    And the "Workout" tab should be active

  @e2e
  Scenario: User can click Library tab and see placeholder
    Given the app is loaded
    When I click on the "Library" tab
    Then the "Library" tab should be active
    And I should see the Library placeholder content

  @e2e
  Scenario: User can switch back to Workout tab
    Given the app is loaded
    And I am on the "Library" tab
    When I click on the "Workout" tab
    Then the "Workout" tab should be active
    And I should see the Workout interface

  @e2e
  Scenario: User can click Analysis tab and see history
    Given the app is loaded
    When I click on the "Analysis" tab
    Then the "Analysis" tab should be active
    And I should see the Analysis history view

  @unit
  Scenario: Tab active state indication
    Given the tab navigation component is rendered
    When the "Workout" tab is active
    Then the "Workout" tab should have active styling
    And the "Library" tab should have inactive styling
    And the "Analysis" tab should have inactive styling

  @unit
  Scenario: Tab click events trigger state changes
    Given the tab navigation component is rendered
    When I click on the "Library" tab
    Then the tab selection state should change to "Library"
    And a state change event should be emitted

  @unit
  Scenario: Tab accessibility attributes
    Given the tab navigation component is rendered
    Then the tab container should have role "tablist"
    And each tab should have role "tab"
    And the active tab should have aria-selected "true"
    And inactive tabs should have aria-selected "false"

  @unit
  Scenario: Gear icon navigates to settings
    Given the app is loaded
    When I tap the gear icon
    Then the settings page should be displayed
    And the tab bar should be hidden

  @unit
  Scenario: Settings page has back arrow
    Given I am on the settings page
    When I tap the back arrow
    Then I should navigate back to the previous tab

  @unit
  Scenario: Three primary tabs with icons
    Given the tab navigation component is rendered
    Then each tab should display an icon and label
    And the tabs should be "Workout", "Library", and "Analysis"
