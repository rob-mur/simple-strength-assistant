//! Pure e1RM (estimated one-rep max) calculation functions.
//!
//! Ported from the Flutter `strength_assistant` reference implementation.
//! No database dependencies or side effects — pure math only.

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
}
