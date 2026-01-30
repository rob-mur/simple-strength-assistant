use serde::{Deserialize, Serialize};

/// Distinguishes between weighted and bodyweight exercises with type safety.
///
/// The enum forces compile-time handling of both exercise types, ensuring
/// that weight-related fields are only present for weighted exercises.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SetType {
    /// A weighted exercise with specific weight configuration
    Weighted {
        /// Current weight for this set
        weight: f32,
        /// Minimum allowed weight for this exercise
        min_weight: f32,
        /// Weight increment (e.g., 2.5kg for standard plates)
        increment: f32,
    },
    /// A bodyweight exercise with no additional weight
    Bodyweight,
}

/// Represents a completed set in a workout.
///
/// Tracks all relevant metrics for a single set, including reps performed,
/// RPE (Rate of Perceived Exertion), and the type of set (weighted or bodyweight).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletedSet {
    /// Sequential set number (1, 2, 3, ...)
    pub set_number: u32,
    /// Number of repetitions completed
    pub reps: u32,
    /// Rate of Perceived Exertion (1.0 to 10.0, in 0.5 increments)
    pub rpe: f32,
    /// Type of set (weighted or bodyweight) with associated data
    pub set_type: SetType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_set_type_pattern_matching() {
        let set_type = SetType::Weighted {
            weight: 100.0,
            min_weight: 20.0,
            increment: 2.5,
        };

        match set_type {
            SetType::Weighted { weight, min_weight, increment } => {
                assert_eq!(weight, 100.0);
                assert_eq!(min_weight, 20.0);
                assert_eq!(increment, 2.5);
            }
            SetType::Bodyweight => panic!("Expected Weighted variant"),
        }
    }

    #[test]
    fn test_bodyweight_set_type_pattern_matching() {
        let set_type = SetType::Bodyweight;

        match set_type {
            SetType::Weighted { .. } => panic!("Expected Bodyweight variant"),
            SetType::Bodyweight => {
                // Success - bodyweight variant matched
            }
        }
    }

    #[test]
    fn test_serde_round_trip_weighted() {
        let original_set = CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 7.5,
            set_type: SetType::Weighted {
                weight: 100.0,
                min_weight: 20.0,
                increment: 2.5,
            },
        };

        let json = serde_json::to_string(&original_set).expect("Serialization failed");
        let deserialized: CompletedSet = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.set_number, original_set.set_number);
        assert_eq!(deserialized.reps, original_set.reps);
        assert_eq!(deserialized.rpe, original_set.rpe);
        assert_eq!(deserialized.set_type, original_set.set_type);
    }

    #[test]
    fn test_serde_round_trip_bodyweight() {
        let original_set = CompletedSet {
            set_number: 2,
            reps: 15,
            rpe: 8.0,
            set_type: SetType::Bodyweight,
        };

        let json = serde_json::to_string(&original_set).expect("Serialization failed");
        let deserialized: CompletedSet = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.set_number, original_set.set_number);
        assert_eq!(deserialized.reps, original_set.reps);
        assert_eq!(deserialized.rpe, original_set.rpe);
        assert_eq!(deserialized.set_type, original_set.set_type);
    }

    #[test]
    fn test_set_type_equality() {
        let weighted1 = SetType::Weighted {
            weight: 100.0,
            min_weight: 20.0,
            increment: 2.5,
        };
        let weighted2 = SetType::Weighted {
            weight: 100.0,
            min_weight: 20.0,
            increment: 2.5,
        };
        let weighted3 = SetType::Weighted {
            weight: 105.0,
            min_weight: 20.0,
            increment: 2.5,
        };

        assert_eq!(weighted1, weighted2);
        assert_ne!(weighted1, weighted3);
        assert_ne!(weighted1, SetType::Bodyweight);
    }
}
