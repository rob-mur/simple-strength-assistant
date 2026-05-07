use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The 12 supported muscle groups for exercise tagging.
///
/// Each variant maps to a TEXT representation in SQLite.
/// The `parent_id` concept (sub-muscle hierarchy) is reserved for v2;
/// all variants return `None` in v1.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MuscleGroup {
    Chest,
    Back,
    Shoulders,
    Biceps,
    Triceps,
    Quads,
    Hamstrings,
    Glutes,
    Calves,
    Core,
    Forearms,
    Traps,
}

impl MuscleGroup {
    /// Returns the parent muscle group for sub-muscle hierarchy.
    /// Always `None` in v1; reserved for future hierarchy support.
    pub fn parent_id(&self) -> Option<MuscleGroup> {
        None
    }
}

impl fmt::Display for MuscleGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MuscleGroup::Chest => "Chest",
            MuscleGroup::Back => "Back",
            MuscleGroup::Shoulders => "Shoulders",
            MuscleGroup::Biceps => "Biceps",
            MuscleGroup::Triceps => "Triceps",
            MuscleGroup::Quads => "Quads",
            MuscleGroup::Hamstrings => "Hamstrings",
            MuscleGroup::Glutes => "Glutes",
            MuscleGroup::Calves => "Calves",
            MuscleGroup::Core => "Core",
            MuscleGroup::Forearms => "Forearms",
            MuscleGroup::Traps => "Traps",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for MuscleGroup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Chest" => Ok(MuscleGroup::Chest),
            "Back" => Ok(MuscleGroup::Back),
            "Shoulders" => Ok(MuscleGroup::Shoulders),
            "Biceps" => Ok(MuscleGroup::Biceps),
            "Triceps" => Ok(MuscleGroup::Triceps),
            "Quads" => Ok(MuscleGroup::Quads),
            "Hamstrings" => Ok(MuscleGroup::Hamstrings),
            "Glutes" => Ok(MuscleGroup::Glutes),
            "Calves" => Ok(MuscleGroup::Calves),
            "Core" => Ok(MuscleGroup::Core),
            "Forearms" => Ok(MuscleGroup::Forearms),
            "Traps" => Ok(MuscleGroup::Traps),
            other => Err(format!("Unknown muscle group: '{}'", other)),
        }
    }
}

/// The contribution tier of a muscle group for a given exercise.
///
/// Tiers map to numeric weights used in volume-load calculations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContributionTier {
    /// The muscle group is the primary mover. Weight: 1.0
    Primary,
    /// The muscle group is a significant secondary mover. Weight: 0.5
    Secondary,
    /// The muscle group provides minor stabilisation. Weight: 0.25
    Tertiary,
}

impl ContributionTier {
    /// Returns the numeric weight used in volume-load calculations.
    pub fn as_weight(&self) -> f64 {
        match self {
            ContributionTier::Primary => 1.0,
            ContributionTier::Secondary => 0.5,
            ContributionTier::Tertiary => 0.25,
        }
    }
}

impl fmt::Display for ContributionTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ContributionTier::Primary => "Primary",
            ContributionTier::Secondary => "Secondary",
            ContributionTier::Tertiary => "Tertiary",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for ContributionTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Primary" => Ok(ContributionTier::Primary),
            "Secondary" => Ok(ContributionTier::Secondary),
            "Tertiary" => Ok(ContributionTier::Tertiary),
            other => Err(format!("Unknown contribution tier: '{}'", other)),
        }
    }
}

/// Associates an exercise with a muscle group and its contribution tier.
///
/// Stored in the `exercise_muscle_groups` join table.
/// The combination of `exercise_id` + `muscle_group` is the primary key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExerciseMuscleGroup {
    /// UUID of the exercise (references `exercises.uuid`)
    pub exercise_id: String,
    /// The muscle group being tagged
    pub muscle_group: MuscleGroup,
    /// How much this muscle group is involved in the exercise
    pub tier: ContributionTier,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── MuscleGroup variant coverage ──────────────────────────────────────────

    #[test]
    fn test_muscle_group_has_exactly_12_variants() {
        let all = [
            MuscleGroup::Chest,
            MuscleGroup::Back,
            MuscleGroup::Shoulders,
            MuscleGroup::Biceps,
            MuscleGroup::Triceps,
            MuscleGroup::Quads,
            MuscleGroup::Hamstrings,
            MuscleGroup::Glutes,
            MuscleGroup::Calves,
            MuscleGroup::Core,
            MuscleGroup::Forearms,
            MuscleGroup::Traps,
        ];
        assert_eq!(all.len(), 12);
    }

    // ── Display (TEXT round-trip) ─────────────────────────────────────────────

    #[test]
    fn test_muscle_group_display_readable_strings() {
        assert_eq!(MuscleGroup::Chest.to_string(), "Chest");
        assert_eq!(MuscleGroup::Back.to_string(), "Back");
        assert_eq!(MuscleGroup::Shoulders.to_string(), "Shoulders");
        assert_eq!(MuscleGroup::Biceps.to_string(), "Biceps");
        assert_eq!(MuscleGroup::Triceps.to_string(), "Triceps");
        assert_eq!(MuscleGroup::Quads.to_string(), "Quads");
        assert_eq!(MuscleGroup::Hamstrings.to_string(), "Hamstrings");
        assert_eq!(MuscleGroup::Glutes.to_string(), "Glutes");
        assert_eq!(MuscleGroup::Calves.to_string(), "Calves");
        assert_eq!(MuscleGroup::Core.to_string(), "Core");
        assert_eq!(MuscleGroup::Forearms.to_string(), "Forearms");
        assert_eq!(MuscleGroup::Traps.to_string(), "Traps");
    }

    #[test]
    fn test_muscle_group_round_trips_through_text() {
        let groups = [
            MuscleGroup::Chest,
            MuscleGroup::Back,
            MuscleGroup::Shoulders,
            MuscleGroup::Biceps,
            MuscleGroup::Triceps,
            MuscleGroup::Quads,
            MuscleGroup::Hamstrings,
            MuscleGroup::Glutes,
            MuscleGroup::Calves,
            MuscleGroup::Core,
            MuscleGroup::Forearms,
            MuscleGroup::Traps,
        ];
        for group in &groups {
            let text = group.to_string();
            let parsed: MuscleGroup = text.parse().expect("round-trip parse failed");
            assert_eq!(&parsed, group, "round-trip failed for {:?}", group);
        }
    }

    #[test]
    fn test_muscle_group_parse_unknown_returns_error() {
        let result = "LatSpread".parse::<MuscleGroup>();
        assert!(result.is_err());
    }

    // ── parent_id always None in v1 ───────────────────────────────────────────

    #[test]
    fn test_muscle_group_parent_id_always_none() {
        let groups = [
            MuscleGroup::Chest,
            MuscleGroup::Back,
            MuscleGroup::Shoulders,
            MuscleGroup::Biceps,
            MuscleGroup::Triceps,
            MuscleGroup::Quads,
            MuscleGroup::Hamstrings,
            MuscleGroup::Glutes,
            MuscleGroup::Calves,
            MuscleGroup::Core,
            MuscleGroup::Forearms,
            MuscleGroup::Traps,
        ];
        for group in &groups {
            assert_eq!(
                group.parent_id(),
                None,
                "parent_id should be None for {:?} in v1",
                group
            );
        }
    }

    // ── ContributionTier weights ──────────────────────────────────────────────

    #[test]
    fn test_contribution_tier_primary_weight_is_1_0() {
        assert_eq!(ContributionTier::Primary.as_weight(), 1.0);
    }

    #[test]
    fn test_contribution_tier_secondary_weight_is_0_5() {
        assert_eq!(ContributionTier::Secondary.as_weight(), 0.5);
    }

    #[test]
    fn test_contribution_tier_tertiary_weight_is_0_25() {
        assert_eq!(ContributionTier::Tertiary.as_weight(), 0.25);
    }

    #[test]
    fn test_contribution_tier_round_trips_through_text() {
        let tiers = [
            ContributionTier::Primary,
            ContributionTier::Secondary,
            ContributionTier::Tertiary,
        ];
        for tier in &tiers {
            let text = tier.to_string();
            let parsed: ContributionTier = text.parse().expect("round-trip parse failed");
            assert_eq!(&parsed, tier, "round-trip failed for {:?}", tier);
        }
    }

    #[test]
    fn test_contribution_tier_parse_unknown_returns_error() {
        let result = "Quaternary".parse::<ContributionTier>();
        assert!(result.is_err());
    }

    // ── ExerciseMuscleGroup struct ────────────────────────────────────────────

    #[test]
    fn test_exercise_muscle_group_construction() {
        let emg = ExerciseMuscleGroup {
            exercise_id: "bench-uuid".to_string(),
            muscle_group: MuscleGroup::Chest,
            tier: ContributionTier::Primary,
        };
        assert_eq!(emg.exercise_id, "bench-uuid");
        assert_eq!(emg.muscle_group, MuscleGroup::Chest);
        assert_eq!(emg.tier, ContributionTier::Primary);
    }

    #[test]
    fn test_exercise_muscle_group_serde_round_trip() {
        let original = ExerciseMuscleGroup {
            exercise_id: "squat-uuid".to_string(),
            muscle_group: MuscleGroup::Quads,
            tier: ContributionTier::Primary,
        };
        let json = serde_json::to_string(&original).expect("serialization failed");
        let deserialized: ExerciseMuscleGroup =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(deserialized, original);
    }

    // ── Domain validation: empty muscle group list ────────────────────────────

    #[test]
    fn test_validate_muscle_groups_rejects_empty_slice() {
        let result = validate_muscle_groups(&[]);
        assert!(
            result.is_err(),
            "empty muscle group list should be rejected"
        );
    }

    #[test]
    fn test_validate_muscle_groups_accepts_non_empty_slice() {
        let groups = vec![ExerciseMuscleGroup {
            exercise_id: "e1".to_string(),
            muscle_group: MuscleGroup::Chest,
            tier: ContributionTier::Primary,
        }];
        assert!(validate_muscle_groups(&groups).is_ok());
    }

    #[test]
    fn test_validate_muscle_groups_accepts_multiple_groups() {
        let groups = vec![
            ExerciseMuscleGroup {
                exercise_id: "e1".to_string(),
                muscle_group: MuscleGroup::Chest,
                tier: ContributionTier::Primary,
            },
            ExerciseMuscleGroup {
                exercise_id: "e1".to_string(),
                muscle_group: MuscleGroup::Triceps,
                tier: ContributionTier::Secondary,
            },
            ExerciseMuscleGroup {
                exercise_id: "e1".to_string(),
                muscle_group: MuscleGroup::Shoulders,
                tier: ContributionTier::Tertiary,
            },
        ];
        assert!(validate_muscle_groups(&groups).is_ok());
    }
}

/// Intensity-adjusted volume for a single muscle group across three time horizons.
///
/// Each field is the sum of per-set contributions where:
/// `contribution = (rpe / 10.0) × (tier_weight / sum_weights_for_exercise)`
///
/// - `daily`: total for sets on today's UTC calendar date
/// - `rolling_7d`: total over the last 7 calendar days (rolling)
/// - `rolling_training_period`: total over the configured training window
#[derive(Debug, Clone, PartialEq)]
pub struct MuscleGroupVolume {
    pub daily: f64,
    pub rolling_7d: f64,
    pub rolling_training_period: f64,
}

/// Domain-level validation: at least one muscle group must be provided.
///
/// Returns `Err` with a descriptive message if `groups` is empty.
/// This check must be applied before calling `Database::set_muscle_groups`.
pub fn validate_muscle_groups(groups: &[ExerciseMuscleGroup]) -> Result<(), String> {
    if groups.is_empty() {
        return Err("At least one muscle group is required for an exercise".to_string());
    }
    Ok(())
}
