/// Three-state progress signal computed from e1RM trend regression.
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressState {
    /// Insufficient session data to run regression.
    InsufficientData,
    /// Linear regression slope > 0: e1RM is trending upward.
    Progressing { slope: f64 },
    /// Linear regression slope ≤ 0: e1RM is flat or declining.
    Stalled { slope: f64 },
}
