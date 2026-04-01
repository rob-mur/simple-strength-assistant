Feature: Edit/Delete Set Modal
  As a user
  I want to edit or delete logged sets in the history view
  So I can correct mistakes or remove unwanted data

  Background:
    Given I have a fresh context and clear storage
    And I create a new database
    And I start a test session with "Squat"
    And I log a set in the current session
    And I finish any active session
    And I navigate directly to the history page

  # AC #1: Tapping a set row opens the edit modal
  Scenario: Tapping a set row opens the edit modal
    When I click on the first set row in the history feed
    Then the edit set modal should be visible
    And it should show "Squat" and "Set 1"

  # AC #2 + #3: Modal is pre-populated and TapeMeasure is configured
  Scenario: Modal is pre-populated with current values
    When I click on the first set row in the history feed
    Then the weight display in the modal should show "0 kg"
    And the reps display in the modal should show "8"
    And the RPE display in the modal should show "7.0"

  # AC #4: Save persists edited values
  Scenario: Saving edited values updates the history feed
    When I click on the first set row in the history feed
    And I change the weight to 5 kg in the modal
    And I change the reps to 9 in the modal
    And I click the save button in the modal
    Then the edit set modal should not be visible
    And the first set row in the history feed should show "5 kg", "9", and "7.0"

  # AC #5 + #6: Delete removes the set and empty groups
  Scenario: Deleting a set removes it from the feed and group disappears if empty
    When I click on the first set row in the history feed
    And I click the delete button in the modal
    Then the edit set modal should not be visible
    And the history feed should be empty

  Scenario: Deleting one set from a group with multiple sets preserves the group
    Given I start a test session with "Squat"
    And I log a set in the current session
    And I finish any active session
    And I navigate directly to the history page
    Then there should be 2 set rows in the history feed
    When I click on the first set row in the history feed
    And I click the delete button in the modal
    Then the edit set modal should not be visible
    And there should be 1 set row in the history feed
    And the "Squat" group should still be visible
