//! Pure e1RM (estimated one-rep max) calculation functions.
//!
//! Ported from the Flutter `strength_assistant` reference implementation.
//! No database dependencies or side effects — pure math only.

use chrono::NaiveDate;

/// Calculates the assumption factor for a given rep count and RPE.
///
/// This represents the fraction of 1RM that can be lifted for the given reps at the given RPE.
pub fn assumption(rep: u32, rpe: f64) -> f64 {
    (rpe * 0.03269803 + 0.6730197) * 0.970546521_f64.powi(rep as i32 - 1)
}

/// Estimates the one-rep max from a completed set.
pub fn e1rm(weight: f64, reps: u32, rpe: f64) -> f64 {
    weight / assumption(reps, rpe)
}

/// Predicts the weight achievable for target reps and RPE given an e1RM.
pub fn predicted_weight(e1rm: f64, target_reps: u32, target_rpe: f64) -> f64 {
    e1rm * assumption(target_reps, target_rpe)
}

/// Computes the least-squares linear regression slope over a series of (date, e1RM) pairs.
///
/// - Input: ordered `(NaiveDate, e1RM)` pairs.
/// - Dates are converted to day-offset indices (days since the first session) for
///   numerical stability; the slope is returned in e1RM units per day.
/// - Returns `0.0` for empty or single-element input (slope undefined).
pub fn e1rm_trend(sessions: &[(NaiveDate, f64)]) -> f64 {
    if sessions.len() < 2 {
        return 0.0;
    }

    let origin = sessions[0].0;
    let n = sessions.len() as f64;

    let (sum_x, sum_y, sum_xx, sum_xy) = sessions.iter().fold(
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
        |(sx, sy, sxx, sxy), (date, e1rm)| {
            let x = (date.signed_duration_since(origin).num_days()) as f64;
            (sx + x, sy + e1rm, sxx + x * x, sxy + x * e1rm)
        },
    );

    let denom = n * sum_xx - sum_x * sum_x;
    if denom == 0.0 {
        return 0.0;
    }

    (n * sum_xy - sum_x * sum_y) / denom
}

/// Blends today's e1RM estimate with a historical average.
///
/// - `factor = 1.0` → returns `today_e1rm` exactly
/// - `factor = 0.0` → returns `historical_e1rm` exactly
/// - `factor = 0.5` → returns midpoint
pub fn blended_e1rm(today_e1rm: f64, historical_e1rm: f64, factor: f64) -> f64 {
    (today_e1rm * factor) + (historical_e1rm * (1.0 - factor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    const EPSILON: f64 = 1e-6;

    // --- assumption() tests ---

    #[test]
    fn assumption_known_inputs_rep5_rpe8() {
        // rep=5, rpe=8: (8 * 0.03269803 + 0.6730197) * 0.970546521^4
        let result = assumption(5, 8.0);
        let expected = (8.0 * 0.03269803 + 0.6730197) * 0.970546521_f64.powi(4);
        assert!(
            (result - expected).abs() < EPSILON,
            "assumption(5, 8.0) = {result}, expected {expected}"
        );
    }

    #[test]
    fn assumption_rep1_rpe10() {
        // rep=1: exponent is 0, so result = rpe * 0.03269803 + 0.6730197
        let result = assumption(1, 10.0);
        let expected = 10.0 * 0.03269803 + 0.6730197;
        assert!((result - expected).abs() < EPSILON);
    }

    #[test]
    fn assumption_rep1_returns_no_nan() {
        let result = assumption(1, 10.0);
        assert!(!result.is_nan());
        assert!(result > 0.0);
    }

    // --- e1rm() tests ---

    #[test]
    fn e1rm_known_values() {
        // 100kg for 5 reps at RPE 8
        let a = assumption(5, 8.0);
        let result = e1rm(100.0, 5, 8.0);
        let expected = 100.0 / a;
        assert!((result - expected).abs() < EPSILON);
    }

    #[test]
    fn e1rm_rep1_rpe10() {
        // At 1 rep RPE 10, e1RM should be close to the actual weight
        let result = e1rm(200.0, 1, 10.0);
        let a = assumption(1, 10.0);
        let expected = 200.0 / a;
        assert!((result - expected).abs() < EPSILON);
        // Should be very close to 200 (assumption at rep=1 rpe=10 is close to 1.0)
        assert!(
            result > 190.0 && result < 210.0,
            "e1rm at rep=1 rpe=10 should be near weight, got {result}"
        );
    }

    // --- predicted_weight() tests ---

    #[test]
    fn predicted_weight_roundtrip() {
        // e1rm from a set, then predict weight for same params → should get original weight back
        let weight = 140.0;
        let reps = 3;
        let rpe = 9.0;
        let estimated = e1rm(weight, reps, rpe);
        let predicted = predicted_weight(estimated, reps, rpe);
        assert!(
            (predicted - weight).abs() < EPSILON,
            "roundtrip failed: predicted={predicted}, original={weight}"
        );
    }

    #[test]
    fn predicted_weight_known_values() {
        let one_rm = 150.0;
        let result = predicted_weight(one_rm, 5, 8.0);
        let expected = one_rm * assumption(5, 8.0);
        assert!((result - expected).abs() < EPSILON);
    }

    // --- blended_e1rm() tests ---

    #[test]
    fn blended_factor_zero_returns_historical() {
        let result = blended_e1rm(120.0, 100.0, 0.0);
        assert!((result - 100.0).abs() < EPSILON);
    }

    #[test]
    fn blended_factor_one_returns_today() {
        let result = blended_e1rm(120.0, 100.0, 1.0);
        assert!((result - 120.0).abs() < EPSILON);
    }

    #[test]
    fn blended_factor_half_returns_midpoint() {
        let result = blended_e1rm(120.0, 100.0, 0.5);
        let expected = 110.0;
        assert!((result - expected).abs() < EPSILON);
    }

    #[test]
    fn blended_same_values_any_factor() {
        // When today == historical, factor doesn't matter
        let result = blended_e1rm(100.0, 100.0, 0.73);
        assert!((result - 100.0).abs() < EPSILON);
    }

    // --- e1rm_trend() tests ---

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn e1rm_trend_empty_returns_zero() {
        assert_eq!(e1rm_trend(&[]), 0.0);
    }

    #[test]
    fn e1rm_trend_single_point_returns_zero() {
        let sessions = [(d(2024, 1, 1), 100.0)];
        assert_eq!(e1rm_trend(&sessions), 0.0);
    }

    #[test]
    fn e1rm_trend_flat_series_returns_zero() {
        let sessions = [
            (d(2024, 1, 1), 100.0),
            (d(2024, 1, 8), 100.0),
            (d(2024, 1, 15), 100.0),
        ];
        let slope = e1rm_trend(&sessions);
        assert!(
            slope.abs() < EPSILON,
            "flat series should give slope ~0.0, got {slope}"
        );
    }

    #[test]
    fn e1rm_trend_positive_slope() {
        // Sessions on day 0, 1, 2 with e1RM 100, 101, 102 → slope = 1.0 kg/day
        let sessions = [
            (d(2024, 1, 1), 100.0),
            (d(2024, 1, 2), 101.0),
            (d(2024, 1, 3), 102.0),
        ];
        let slope = e1rm_trend(&sessions);
        assert!(
            (slope - 1.0).abs() < EPSILON,
            "expected slope 1.0, got {slope}"
        );
    }

    #[test]
    fn e1rm_trend_negative_slope() {
        // Sessions on day 0, 1, 2 with e1RM 102, 101, 100 → slope = -1.0 kg/day
        let sessions = [
            (d(2024, 1, 1), 102.0),
            (d(2024, 1, 2), 101.0),
            (d(2024, 1, 3), 100.0),
        ];
        let slope = e1rm_trend(&sessions);
        assert!(
            (slope - (-1.0)).abs() < EPSILON,
            "expected slope -1.0, got {slope}"
        );
    }

    #[test]
    fn e1rm_trend_date_conversion_stability_over_large_range() {
        // 365-day span: day 0 = 100.0, day 365 = 165.0
        // slope = (165.0 - 100.0) / 365 ≈ 0.17808... kg/day
        let start = d(2023, 1, 1);
        let end = d(2024, 1, 1); // exactly 365 days
        let sessions = [(start, 100.0), (end, 165.0)];
        let slope = e1rm_trend(&sessions);
        let expected = 65.0 / 365.0;
        assert!(
            (slope - expected).abs() < EPSILON,
            "expected slope {expected}, got {slope}"
        );
    }

    #[test]
    fn e1rm_trend_output_is_finite() {
        // Realistic input: 500 sessions over ~5 years, weights 60–200 kg
        let start = d(2020, 1, 1);
        let sessions: Vec<(NaiveDate, f64)> = (0..500)
            .map(|i| {
                let date = start + Duration::days(i * 3);
                let weight = 60.0 + (i as f64) * 0.28;
                (date, weight)
            })
            .collect();
        let slope = e1rm_trend(&sessions);
        assert!(slope.is_finite(), "slope should be finite, got {slope}");
    }
}
