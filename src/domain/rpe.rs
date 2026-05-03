/// Map a numeric RPE value to a qualitative description.
///
/// The scale follows standard RPE conventions:
/// - 10: Max effort, no reps left in reserve
/// - 9–9.5: Very Hard, 0–1 rep in reserve
/// - 7.5–8.5: Hard / Challenging
/// - 6–7: Moderate / Solid
/// - Below 6: Light / Warmup
pub fn rpe_description(rpe: f64) -> &'static str {
    match rpe {
        v if v >= 10.0 => "Max",
        v if v >= 9.5 => "Near Max",
        v if v >= 9.0 => "Very Hard",
        v if v >= 8.5 => "Very Hard",
        v if v >= 8.0 => "Hard",
        v if v >= 7.5 => "Challenging",
        v if v >= 7.0 => "Moderate",
        v if v >= 6.0 => "Solid",
        v if v >= 5.0 => "Light",
        v if v >= 4.0 => "Easy",
        _ => "Warmup",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_effort() {
        assert_eq!(rpe_description(10.0), "Max");
    }

    #[test]
    fn near_max() {
        assert_eq!(rpe_description(9.5), "Near Max");
    }

    #[test]
    fn very_hard() {
        assert_eq!(rpe_description(9.0), "Very Hard");
        assert_eq!(rpe_description(8.5), "Very Hard");
    }

    #[test]
    fn hard() {
        assert_eq!(rpe_description(8.0), "Hard");
    }

    #[test]
    fn challenging() {
        assert_eq!(rpe_description(7.5), "Challenging");
    }

    #[test]
    fn moderate() {
        assert_eq!(rpe_description(7.0), "Moderate");
    }

    #[test]
    fn solid() {
        assert_eq!(rpe_description(6.0), "Solid");
        assert_eq!(rpe_description(6.5), "Solid");
    }

    #[test]
    fn light() {
        assert_eq!(rpe_description(5.0), "Light");
        assert_eq!(rpe_description(5.5), "Light");
    }

    #[test]
    fn easy() {
        assert_eq!(rpe_description(4.0), "Easy");
        assert_eq!(rpe_description(4.5), "Easy");
    }

    #[test]
    fn warmup() {
        assert_eq!(rpe_description(3.0), "Warmup");
        assert_eq!(rpe_description(1.0), "Warmup");
    }

    #[test]
    fn above_max_still_returns_max() {
        assert_eq!(rpe_description(11.0), "Max");
    }
}
