@fast
Feature: Pull-to-refresh disabled at the document root (Issue 188)
  As a user dragging horizontal sliders on a mobile PWA
  I want the document root to suppress pull-to-refresh
  So that vertical drift during a slider drag does not reload the page

  Background:
    Given I have a fresh context and clear storage
    And I create a new database

  Scenario: Document root declares overscroll-behavior-y that suppresses pull-to-refresh
    Then the document root should declare overscroll-behavior-y of contain or none

  Scenario: RPE slider drag survives downward vertical drift on mobile viewport
    Given I start a test session with "Bench Drag"
    When I drag the RPE slider thumb horizontally with downward vertical drift
    Then the RPE slider value should have changed from its initial value
