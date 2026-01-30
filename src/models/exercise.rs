use serde::{Deserialize, Serialize};

/// Configuration for the type of set an exercise uses.
///
/// This is separate from `SetType` because it represents the exercise's
/// configuration (which doesn't include actual weight), whereas `SetType`
/// represents a completed set (which includes the weight used).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SetTypeConfig {
    /// Configuration for weighted exercises
    Weighted {
        /// Minimum allowed weight for this exercise
        min_weight: f32,
        /// Weight increment (e.g., 2.5kg for standard plates)
        increment: f32,
    },
    /// Configuration for bodyweight exercises
    Bodyweight,
}

/// Metadata describing an exercise and its configuration.
///
/// Contains the exercise name and the type of sets it uses,
/// which determines what fields are tracked for each set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExerciseMetadata {
    /// Display name of the exercise (e.g., "Bench Press", "Pull-ups")
    pub name: String,
    /// Configuration for the type of sets this exercise uses
    pub set_type_config: SetTypeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_exercise_metadata() {
        let exercise = ExerciseMetadata {
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
        };

        assert_eq!(exercise.name, "Bench Press");
        match exercise.set_type_config {
            SetTypeConfig::Weighted { min_weight, increment } => {
                assert_eq!(min_weight, 20.0);
                assert_eq!(increment, 2.5);
            }
            SetTypeConfig::Bodyweight => panic!("Expected Weighted config"),
        }
    }

    #[test]
    fn test_bodyweight_exercise_metadata() {
        let exercise = ExerciseMetadata {
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
        };

        assert_eq!(exercise.name, "Pull-ups");
        match exercise.set_type_config {
            SetTypeConfig::Bodyweight => {
                // Success - bodyweight config matched
            }
            SetTypeConfig::Weighted { .. } => panic!("Expected Bodyweight config"),
        }
    }

    #[test]
    fn test_serde_round_trip_weighted_exercise() {
        let original = ExerciseMetadata {
            name: "Squat".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
        };

        let json = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: ExerciseMetadata = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.set_type_config, original.set_type_config);
    }

    #[test]
    fn test_serde_round_trip_bodyweight_exercise() {
        let original = ExerciseMetadata {
            name: "Push-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
        };

        let json = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: ExerciseMetadata = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.set_type_config, original.set_type_config);
    }

    #[test]
    fn test_set_type_config_equality() {
        let weighted1 = SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        };
        let weighted2 = SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        };
        let weighted3 = SetTypeConfig::Weighted {
            min_weight: 25.0,
            increment: 2.5,
        };

        assert_eq!(weighted1, weighted2);
        assert_ne!(weighted1, weighted3);
        assert_ne!(weighted1, SetTypeConfig::Bodyweight);
    }

    #[test]
    fn test_exercise_cloning() {
        let original = ExerciseMetadata {
            name: "Deadlift".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
        };

        let cloned = original.clone();
        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.set_type_config, original.set_type_config);
    }
}
