use serde::{Deserialize, Serialize};

/// Global application settings, stored as a single row in the `settings` table.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    /// Target RPE for auto-programming (default: 8.0)
    pub target_rpe: f64,
    /// Number of days of history to consider (default: 30)
    pub history_window_days: i32,
    /// Blend factor for today's session vs. historical data (default: 0.5)
    pub today_blend_factor: f64,
    /// Default number of planned sets when adding an exercise to a plan (default: 3)
    pub default_planned_sets: u32,
    /// Default rep count suggested for bodyweight exercises (default: 10)
    pub default_bodyweight_reps: u32,
    /// Minimum number of training sessions containing an exercise before a
    /// Progress State is emitted (default: 3)
    pub min_sessions_for_regression: i64,
    /// Rolling lookback window for progress detection, in weeks (default: 12)
    pub training_window_weeks: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_rpe: 8.0,
            history_window_days: 30,
            today_blend_factor: 0.5,
            default_planned_sets: 3,
            default_bodyweight_reps: 10,
            min_sessions_for_regression: 3,
            training_window_weeks: 12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let s = Settings::default();
        assert_eq!(s.target_rpe, 8.0);
        assert_eq!(s.history_window_days, 30);
        assert_eq!(s.today_blend_factor, 0.5);
        assert_eq!(s.default_planned_sets, 3);
        assert_eq!(s.default_bodyweight_reps, 10);
        assert_eq!(s.min_sessions_for_regression, 3);
        assert_eq!(s.training_window_weeks, 12);
    }

    #[test]
    fn test_settings_serde_round_trip() {
        let original = Settings {
            target_rpe: 7.5,
            history_window_days: 14,
            today_blend_factor: 0.3,
            default_planned_sets: 5,
            default_bodyweight_reps: 15,
            min_sessions_for_regression: 5,
            training_window_weeks: 8,
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let deserialized: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_settings_progress_detection_defaults() {
        let s = Settings::default();
        // min_sessions_for_regression must be at least 1 (guards against zero-division)
        assert!(s.min_sessions_for_regression >= 1);
        // training_window_weeks must be positive
        assert!(s.training_window_weeks > 0);
    }
}
