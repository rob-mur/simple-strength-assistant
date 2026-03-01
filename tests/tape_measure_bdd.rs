use cucumber::World;

mod steps;

// Constants from TapeMeasure component
const PIXELS_PER_STEP: f64 = 60.0;
const FRICTION: f64 = 0.85;
const VELOCITY_THRESHOLD: f64 = 0.5;
const SNAP_STIFFNESS: f64 = 0.25;

#[derive(Debug, Default, World)]
pub struct TapeMeasureWorld {
    // Component state
    pub offset: f64,
    pub velocity: f64,
    pub is_dragging: bool,
    pub is_snapping: bool,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,

    // Interaction state
    pub last_pointer_x: f64,
    pub pointer_captured: bool,

    // Test tracking
    pub initial_offset: f64,
}

impl TapeMeasureWorld {
    /// Initialize with default TapeMeasure configuration
    pub fn init_with_defaults(&mut self) {
        self.value = 100.0;
        self.min = 0.0;
        self.max = 300.0;
        self.step = 2.5;
        self.offset = (self.value - self.min) / self.step * -PIXELS_PER_STEP;
        self.velocity = 0.0;
        self.is_dragging = false;
        self.is_snapping = false;
        self.pointer_captured = false;
    }

    /// Initialize with specific value
    pub fn init_with_value(&mut self, value: f64) {
        self.init_with_defaults();
        self.value = value;
        self.offset = (self.value - self.min) / self.step * -PIXELS_PER_STEP;
    }

    /// Simulate pointer down event
    pub fn pointer_down(&mut self, x: f64) {
        self.is_dragging = true;
        self.is_snapping = false;
        self.last_pointer_x = x;
        self.velocity = 0.0;
        self.pointer_captured = true;
    }

    /// Simulate pointer move event
    pub fn pointer_move(&mut self, x: f64) {
        if self.is_dragging {
            let delta_x = x - self.last_pointer_x;
            self.offset += delta_x;

            // Apply edge clamping
            let total_steps = (self.max - self.min) / self.step;
            let min_offset = total_steps * -PIXELS_PER_STEP;
            let max_offset = 0.0;

            if self.offset > max_offset {
                self.offset = max_offset;
            } else if self.offset < min_offset {
                self.offset = min_offset;
            }

            self.last_pointer_x = x;
        }
    }

    /// Simulate pointer up event with optional velocity
    pub fn pointer_up(&mut self, velocity: Option<f64>) {
        self.is_dragging = false;
        self.pointer_captured = false;

        if let Some(v) = velocity {
            self.velocity = v;
            if v.abs() < VELOCITY_THRESHOLD {
                self.is_snapping = true;
            }
        } else {
            self.is_snapping = true;
        }
    }

    /// Simulate one frame of physics (momentum or snapping)
    pub fn tick_physics(&mut self) {
        if self.is_dragging {
            return;
        }

        if self.velocity.abs() >= VELOCITY_THRESHOLD {
            // Momentum phase
            self.velocity *= FRICTION;
            self.offset += self.velocity;

            // Edge clamping
            let total_steps = (self.max - self.min) / self.step;
            let min_offset = total_steps * -PIXELS_PER_STEP;
            let max_offset = 0.0;

            if self.offset > max_offset {
                self.offset = max_offset;
                self.velocity = 0.0;
            } else if self.offset < min_offset {
                self.offset = min_offset;
                self.velocity = 0.0;
            }

            // Trigger snap when velocity falls below threshold
            if self.velocity.abs() < VELOCITY_THRESHOLD {
                self.velocity = 0.0;
                self.is_snapping = true;
            }
        } else if self.is_snapping {
            // Snapping phase
            let target_offset = (self.offset / PIXELS_PER_STEP).round() * PIXELS_PER_STEP;
            let diff = target_offset - self.offset;

            if diff.abs() > 0.01 {
                self.offset += diff * SNAP_STIFFNESS;
            } else {
                self.offset = target_offset;
                self.velocity = 0.0;
                self.is_snapping = false;
            }
        }
    }

    /// Update value from external source (prop change)
    pub fn update_value(&mut self, new_value: f64) {
        if !self.is_dragging && !self.is_snapping {
            self.value = new_value;
            self.offset = (self.value - self.min) / self.step * -PIXELS_PER_STEP;
            self.velocity = 0.0;
        }
    }

    /// Calculate current value from offset
    pub fn current_value(&self) -> f64 {
        self.min + (self.offset / -PIXELS_PER_STEP) * self.step
    }
}

#[tokio::test]
async fn run_cucumber_tests() {
    TapeMeasureWorld::cucumber().run("tests/features").await;
}
