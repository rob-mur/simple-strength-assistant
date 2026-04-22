use serde::{Deserialize, Serialize};

/// Configuration for the type of set an exercise uses.
///
/// This is separate from `SetType` because it represents the exercise's
/// configuration (which doesn't include actual weight), whereas `SetType`
/// represents a completed set (which includes the weight used).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct ExerciseMetadata {
    /// Optional database ID for the exercise (UUID string)
    pub id: Option<String>,
    /// Display name of the exercise (e.g., "Bench Press", "Pull-ups")
    pub name: String,
    /// Configuration for the type of sets this exercise uses
    pub set_type_config: SetTypeConfig,
    /// Minimum number of reps for this exercise (default: 1)
    #[serde(default = "default_min_reps")]
    pub min_reps: i32,
    /// Maximum number of reps for this exercise (None = unlimited)
    #[serde(default)]
    pub max_reps: Option<i32>,
}

fn default_min_reps() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_exercise_metadata() {
        let exercise = ExerciseMetadata {
            id: None,
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
            min_reps: 1,
            max_reps: None,
        };

        assert_eq!(exercise.name, "Bench Press");
        match exercise.set_type_config {
            SetTypeConfig::Weighted {
                min_weight,
                increment,
            } => {
                assert_eq!(min_weight, 20.0);
                assert_eq!(increment, 2.5);
            }
            SetTypeConfig::Bodyweight => panic!("Expected Weighted config"),
        }
    }

    #[test]
    fn test_bodyweight_exercise_metadata() {
        let exercise = ExerciseMetadata {
            id: None,
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
            min_reps: 1,
            max_reps: None,
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
            id: Some("test-uuid-123".to_string()),
            name: "Squat".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
            min_reps: 1,
            max_reps: None,
        };

        let json = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: ExerciseMetadata =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.set_type_config, original.set_type_config);
    }

    #[test]
    fn test_serde_round_trip_bodyweight_exercise() {
        let original = ExerciseMetadata {
            id: None,
            name: "Push-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
            min_reps: 1,
            max_reps: None,
        };

        let json = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: ExerciseMetadata =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.set_type_config, original.set_type_config);
    }

    #[test]
    fn test_exercise_cloning() {
        let original = ExerciseMetadata {
            id: Some("test-uuid-99".to_string()),
            name: "Deadlift".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
            min_reps: 1,
            max_reps: None,
        };

        let cloned = original.clone();
        assert_eq!(cloned.id, original.id);
        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.set_type_config, original.set_type_config);
    }

    #[test]
    fn test_serde_round_trip_with_rep_range() {
        let original = ExerciseMetadata {
            id: Some("test-uuid-42".to_string()),
            name: "Squat".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
            min_reps: 3,
            max_reps: Some(8),
        };

        let json = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: ExerciseMetadata =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.min_reps, 3);
        assert_eq!(deserialized.max_reps, Some(8));
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_serde_defaults_for_missing_rep_fields() {
        // Simulate JSON from older version without rep range fields
        let json = r#"{"id":"uuid-1","name":"Bench","set_type_config":{"Weighted":{"min_weight":20.0,"increment":2.5}}}"#;
        let deserialized: ExerciseMetadata =
            serde_json::from_str(json).expect("Deserialization failed");

        assert_eq!(deserialized.min_reps, 1);
        assert_eq!(deserialized.max_reps, None);
    }
}
