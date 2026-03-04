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

  Scenario: External value changes update tape position
    Given the TapeMeasure is initialized with value 100kg
    When the parent component updates the value prop to 150kg
    Then the tape should immediately scroll to center 150kg
    And the offset should reflect the new position
    And velocity should be reset to 0.0

  Scenario: Scale change while idle resets tape position correctly
    Given the TapeMeasure is initialized with value 100, step 1, min 0
    When the parent changes step to 0.5 and min to 0
    Then the tape should reposition to center value 100 at the new scale
    And velocity should be 0.0

  Scenario: Scale change followed by interaction uses new scale
    Given the TapeMeasure is initialized with value 10, step 2.5, min 0
    When the parent changes step to 5.0 and min to 0
    And the tape measure enters snapping state
    Then the component should snap to a multiple of 5.0


