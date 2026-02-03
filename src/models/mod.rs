/// Core data models and type system for the strength training application.
///
/// This module provides type-safe data structures for representing exercises,
/// sets, and workout data, along with validation logic to ensure data integrity.
pub mod exercise;
pub mod set;
pub mod validation;

// Re-export commonly used types for easier access
// Allow unused imports as these are re-exported for public use by consumers of this module
#[allow(unused_imports)]
pub use exercise::{ExerciseMetadata, SetTypeConfig};
#[allow(unused_imports)]
pub use set::{CompletedSet, SetType};
#[allow(unused_imports)]
pub use validation::{
    ValidationError, validate_completed_set, validate_reps, validate_rpe, validate_set_number,
    validate_weight,
};
