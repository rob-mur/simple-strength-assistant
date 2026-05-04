@fast
Feature: Mobile recording view fits without vertical scroll
  As a mobile user on an iPhone SE (375×667)
  I want the workout recording surface to fit entirely within the viewport
  So that I never need to scroll to reach essential controls

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Recording view has no vertical overflow on mobile
    Given I start a plan-based session with "Bench Press"
    And I log a set in the current session
    Then the page should have no vertical scroll overflow
    And the Weight input should render exactly two rows
    And the Reps input should render exactly two rows
    And the LOG SET button should be visible within the viewport
    And the action menu trigger should be visible within the viewport
