use serde::{Deserialize, Serialize};

/// Global application settings, stored as a single row in the `settings` table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    /// Target RPE for auto-programming (default: 8.0)
    pub target_rpe: f64,
    /// Number of days of history to consider (default: 30)
    pub history_window_days: i32,
    /// Blend factor for today's session vs. historical data (default: 0.5)
    pub today_blend_factor: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_rpe: 8.0,
            history_window_days: 30,
            today_blend_factor: 0.5,
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
    }

    #[test]
    fn test_settings_serde_round_trip() {
        let original = Settings {
            target_rpe: 7.5,
            history_window_days: 14,
            today_blend_factor: 0.3,
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let deserialized: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, original);
    }
}
