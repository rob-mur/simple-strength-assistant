use super::exercise::{ExerciseMetadata, SetTypeConfig};
use super::set::{CompletedSet, SetType};

/// Validation errors that can occur when validating exercise data.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ValidationError {
    /// Weight is below the minimum allowed for this exercise
    WeightBelowMinimum { weight: f32, min_weight: f32 },
    /// Weight exceeds the maximum sanity check limit
    WeightExceedsMaximum { weight: f32, max_weight: f32 },
    /// Weight is not a valid multiple of the increment
    WeightNotMultipleOfIncrement { weight: f32, increment: f32 },
    /// RPE is outside the valid range (1.0 to 10.0)
    RpeOutOfBounds { rpe: f32 },
    /// RPE is not a valid 0.5 increment
    RpeInvalidStep { rpe: f32 },
    /// Number of reps is zero
    ZeroReps,
    /// Number of reps exceeds sanity check limit
    RepsExceedLimit { reps: u32, limit: u32 },
    /// Set number is zero
    ZeroSetNumber,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::WeightBelowMinimum { weight, min_weight } => {
                write!(
                    f,
                    "Weight {:.1}kg is below minimum {:.1}kg",
                    weight, min_weight
                )
            }
            ValidationError::WeightExceedsMaximum { weight, max_weight } => {
                write!(
                    f,
                    "Weight {:.1}kg exceeds maximum sanity check limit {:.1}kg",
                    weight, max_weight
                )
            }
            ValidationError::WeightNotMultipleOfIncrement { weight, increment } => {
                write!(
                    f,
                    "Weight {:.1}kg is not a multiple of increment {:.1}kg",
                    weight, increment
                )
            }
            ValidationError::RpeOutOfBounds { rpe } => {
                write!(f, "RPE {:.1} is outside valid range (1.0 to 10.0)", rpe)
            }
            ValidationError::RpeInvalidStep { rpe } => {
                write!(f, "RPE {:.1} must be in 0.5 increments", rpe)
            }
            ValidationError::ZeroReps => {
                write!(f, "Number of reps must be greater than 0")
            }
            ValidationError::RepsExceedLimit { reps, limit } => {
                write!(
                    f,
                    "Number of reps ({}) exceeds sanity check limit ({})",
                    reps, limit
                )
            }
            ValidationError::ZeroSetNumber => {
                write!(f, "Set number must be greater than 0")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates that a weight is at or above the minimum and is a valid multiple of the increment.
///
/// # Arguments
/// * `weight` - The weight to validate
/// * `min_weight` - The minimum allowed weight
/// * `increment` - The weight increment (e.g., 2.5kg)
///
/// # Returns
/// `Ok(())` if valid, otherwise a `ValidationError`
#[allow(dead_code)]
pub fn validate_weight(
    weight: f32,
    min_weight: f32,
    increment: f32,
) -> Result<(), ValidationError> {
    // Maximum sanity check - 500kg should be more than enough for any human exercise
    const MAX_WEIGHT: f32 = 500.0;

    if weight < min_weight {
        return Err(ValidationError::WeightBelowMinimum { weight, min_weight });
    }

    if weight > MAX_WEIGHT {
        return Err(ValidationError::WeightExceedsMaximum {
            weight,
            max_weight: MAX_WEIGHT,
        });
    }

    // Check if weight is a valid multiple of increment relative to min_weight
    let diff = weight - min_weight;
    let remainder = (diff / increment).fract().abs();

    // Use a small epsilon for floating point comparison
    // Accept values very close to 0.0 (exact multiple) or 1.0 (rounding to next multiple)
    const EPSILON: f32 = 0.001;
    if remainder > EPSILON && remainder < (1.0 - EPSILON) {
        return Err(ValidationError::WeightNotMultipleOfIncrement { weight, increment });
    }

    Ok(())
}

/// Validates that RPE is within bounds (1.0 to 10.0) and is a valid 0.5 increment.
///
/// # Arguments
/// * `rpe` - The Rate of Perceived Exertion to validate
///
/// # Returns
/// `Ok(())` if valid, otherwise a `ValidationError`
#[allow(dead_code)]
pub fn validate_rpe(rpe: f32) -> Result<(), ValidationError> {
    if !(1.0..=10.0).contains(&rpe) {
        return Err(ValidationError::RpeOutOfBounds { rpe });
    }

    // Check if RPE is a valid 0.5 increment
    let remainder = (rpe * 2.0).fract().abs();
    if remainder > 0.001 {
        return Err(ValidationError::RpeInvalidStep { rpe });
    }

    Ok(())
}

/// Validates that the number of reps is positive and within reasonable bounds.
///
/// # Arguments
/// * `reps` - The number of repetitions to validate
///
/// # Returns
/// `Ok(())` if valid, otherwise a `ValidationError`
#[allow(dead_code)]
pub fn validate_reps(reps: u32) -> Result<(), ValidationError> {
    const MAX_REPS: u32 = 100;

    if reps == 0 {
        return Err(ValidationError::ZeroReps);
    }

    if reps > MAX_REPS {
        return Err(ValidationError::RepsExceedLimit {
            reps,
            limit: MAX_REPS,
        });
    }

    Ok(())
}

/// Validates that the set number is positive.
///
/// # Arguments
/// * `set_number` - The set number to validate
///
/// # Returns
/// `Ok(())` if valid, otherwise a `ValidationError`
#[allow(dead_code)]
pub fn validate_set_number(set_number: u32) -> Result<(), ValidationError> {
    if set_number == 0 {
        return Err(ValidationError::ZeroSetNumber);
    }

    Ok(())
}

/// Validates a complete set, checking all fields according to their respective rules.
///
/// # Arguments
/// * `set` - The completed set to validate
/// * `exercise` - The exercise metadata containing validation constraints
///
/// # Returns
/// `Ok(())` if all validations pass, otherwise the first `ValidationError` encountered
#[allow(dead_code)]
pub fn validate_completed_set(
    set: &CompletedSet,
    exercise: &ExerciseMetadata,
) -> Result<(), ValidationError> {
    validate_set_number(set.set_number)?;
    validate_reps(set.reps)?;
    validate_rpe(set.rpe)?;

    // Validate weight if this is a weighted set
    match (&set.set_type, &exercise.set_type_config) {
        (
            SetType::Weighted { weight },
            SetTypeConfig::Weighted {
                min_weight,
                increment,
            },
        ) => {
            validate_weight(*weight, *min_weight, *increment)?;
        }
        (SetType::Bodyweight, SetTypeConfig::Bodyweight) => {
            // Valid combination - no weight to validate
        }
        (SetType::Weighted { .. }, SetTypeConfig::Bodyweight) => {
            // This shouldn't happen - weighted set for bodyweight exercise
            // For now we'll allow it, but could add a validation error variant for this
        }
        (SetType::Bodyweight, SetTypeConfig::Weighted { .. }) => {
            // This shouldn't happen - bodyweight set for weighted exercise
            // For now we'll allow it, but could add a validation error variant for this
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_weight_below_minimum() {
        let result = validate_weight(15.0, 20.0, 2.5);
        assert_eq!(
            result,
            Err(ValidationError::WeightBelowMinimum {
                weight: 15.0,
                min_weight: 20.0
            })
        );
    }

    #[test]
    fn test_validate_weight_at_minimum() {
        let result = validate_weight(20.0, 20.0, 2.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_weight_valid_increment() {
        let result = validate_weight(25.0, 20.0, 2.5);
        assert!(result.is_ok());

        let result = validate_weight(27.5, 20.0, 2.5);
        assert!(result.is_ok());

        let result = validate_weight(100.0, 20.0, 2.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_weight_not_multiple_of_increment() {
        let result = validate_weight(21.0, 20.0, 2.5);
        assert_eq!(
            result,
            Err(ValidationError::WeightNotMultipleOfIncrement {
                weight: 21.0,
                increment: 2.5
            })
        );

        let result = validate_weight(23.75, 20.0, 2.5);
        assert_eq!(
            result,
            Err(ValidationError::WeightNotMultipleOfIncrement {
                weight: 23.75,
                increment: 2.5
            })
        );
    }

    #[test]
    fn test_validate_weight_exceeds_maximum() {
        let result = validate_weight(600.0, 20.0, 2.5);
        assert_eq!(
            result,
            Err(ValidationError::WeightExceedsMaximum {
                weight: 600.0,
                max_weight: 500.0
            })
        );
    }

    #[test]
    fn test_validate_rpe_valid_values() {
        // Test all valid 0.5 increments
        for i in 2..=20 {
            let rpe = i as f32 / 2.0;
            let result = validate_rpe(rpe);
            assert!(result.is_ok(), "RPE {} should be valid", rpe);
        }
    }

    #[test]
    fn test_validate_rpe_out_of_bounds() {
        let result = validate_rpe(0.5);
        assert_eq!(result, Err(ValidationError::RpeOutOfBounds { rpe: 0.5 }));

        let result = validate_rpe(10.5);
        assert_eq!(result, Err(ValidationError::RpeOutOfBounds { rpe: 10.5 }));

        let result = validate_rpe(0.0);
        assert_eq!(result, Err(ValidationError::RpeOutOfBounds { rpe: 0.0 }));

        let result = validate_rpe(11.0);
        assert_eq!(result, Err(ValidationError::RpeOutOfBounds { rpe: 11.0 }));
    }

    #[test]
    fn test_validate_rpe_invalid_step() {
        let result = validate_rpe(7.3);
        assert_eq!(result, Err(ValidationError::RpeInvalidStep { rpe: 7.3 }));

        let result = validate_rpe(8.7);
        assert_eq!(result, Err(ValidationError::RpeInvalidStep { rpe: 8.7 }));

        let result = validate_rpe(5.1);
        assert_eq!(result, Err(ValidationError::RpeInvalidStep { rpe: 5.1 }));
    }

    #[test]
    fn test_validate_zero_reps() {
        let result = validate_reps(0);
        assert_eq!(result, Err(ValidationError::ZeroReps));
    }

    #[test]
    fn test_validate_valid_reps() {
        assert!(validate_reps(1).is_ok());
        assert!(validate_reps(5).is_ok());
        assert!(validate_reps(10).is_ok());
        assert!(validate_reps(50).is_ok());
        assert!(validate_reps(100).is_ok());
    }

    #[test]
    fn test_validate_reps_exceed_limit() {
        let result = validate_reps(101);
        assert_eq!(
            result,
            Err(ValidationError::RepsExceedLimit {
                reps: 101,
                limit: 100
            })
        );

        let result = validate_reps(200);
        assert_eq!(
            result,
            Err(ValidationError::RepsExceedLimit {
                reps: 200,
                limit: 100
            })
        );
    }

    #[test]
    fn test_validate_zero_set_number() {
        let result = validate_set_number(0);
        assert_eq!(result, Err(ValidationError::ZeroSetNumber));
    }

    #[test]
    fn test_validate_valid_set_number() {
        assert!(validate_set_number(1).is_ok());
        assert!(validate_set_number(5).is_ok());
        assert!(validate_set_number(100).is_ok());
    }

    #[test]
    fn test_validate_completed_set_weighted_valid() {
        let exercise = ExerciseMetadata {
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
        };

        let set = CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 7.5,
            set_type: SetType::Weighted { weight: 100.0 },
        };

        assert!(validate_completed_set(&set, &exercise).is_ok());
    }

    #[test]
    fn test_validate_completed_set_bodyweight_valid() {
        let exercise = ExerciseMetadata {
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
        };

        let set = CompletedSet {
            set_number: 2,
            reps: 15,
            rpe: 8.0,
            set_type: SetType::Bodyweight,
        };

        assert!(validate_completed_set(&set, &exercise).is_ok());
    }

    #[test]
    fn test_validate_completed_set_invalid_weight() {
        let exercise = ExerciseMetadata {
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
        };

        let set = CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 7.5,
            set_type: SetType::Weighted { weight: 15.0 },
        };

        let result = validate_completed_set(&set, &exercise);
        assert_eq!(
            result,
            Err(ValidationError::WeightBelowMinimum {
                weight: 15.0,
                min_weight: 20.0
            })
        );
    }

    #[test]
    fn test_validate_completed_set_invalid_rpe() {
        let exercise = ExerciseMetadata {
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
        };

        let set = CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 11.0,
            set_type: SetType::Bodyweight,
        };

        let result = validate_completed_set(&set, &exercise);
        assert_eq!(result, Err(ValidationError::RpeOutOfBounds { rpe: 11.0 }));
    }

    #[test]
    fn test_validate_completed_set_zero_reps() {
        let exercise = ExerciseMetadata {
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
        };

        let set = CompletedSet {
            set_number: 1,
            reps: 0,
            rpe: 7.0,
            set_type: SetType::Bodyweight,
        };

        let result = validate_completed_set(&set, &exercise);
        assert_eq!(result, Err(ValidationError::ZeroReps));
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::WeightBelowMinimum {
            weight: 15.0,
            min_weight: 20.0,
        };
        assert_eq!(format!("{}", err), "Weight 15.0kg is below minimum 20.0kg");

        let err = ValidationError::WeightExceedsMaximum {
            weight: 600.0,
            max_weight: 500.0,
        };
        assert_eq!(
            format!("{}", err),
            "Weight 600.0kg exceeds maximum sanity check limit 500.0kg"
        );

        let err = ValidationError::RpeOutOfBounds { rpe: 11.0 };
        assert_eq!(
            format!("{}", err),
            "RPE 11.0 is outside valid range (1.0 to 10.0)"
        );

        let err = ValidationError::ZeroReps;
        assert_eq!(format!("{}", err), "Number of reps must be greater than 0");
    }
}
