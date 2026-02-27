Feature: TapeMeasure Physics (Momentum and Snapping)

  Scenario: Momentum glide
    Given the TapeMeasure component is dragging
    When I release the pointer at a velocity of 100 units/frame
    Then the component should continue to glide with decaying velocity
    And the internal offset should continue to change after release

  Scenario: Snapping
    Given the TapeMeasure component is gliding
    When its velocity falls below the threshold
    Then it should interpolate toward the nearest step increment
    And the final offset should be an exact multiple of the step width

  Scenario: Edge resistance
    Given the TapeMeasure is at its minimum value
    When I try to drag it past the boundary
    Then the offset should be clamped or offer resistance
    And the value should not go below the minimum

  Scenario: Tap to stop
    Given the TapeMeasure is gliding
    When I press down on the component
    Then the glide should immediately stop
    And the component should snap to the current nearest increment
