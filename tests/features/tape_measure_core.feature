Feature: TapeMeasure Core Interaction

  Scenario: Smooth dragging
    Given the TapeMeasure component is rendered
    When I press down on the component at X 100
    And I move the pointer to X 150
    Then the internal offset should increase by 50 units

  Scenario: Scroll locking
    Given the TapeMeasure component is rendered
    Then the component container should have "touch-action: none" style
    And browser default scrolling should be prevented during drag

  Scenario: Pointer capture
    Given the TapeMeasure component is rendered
    When I press down on the component
    Then the component should capture the pointer
    When I move the pointer outside the component boundaries
    Then the component should still receive pointer move events
