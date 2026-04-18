/// Pairing flow state machine.
#[derive(Clone, Debug, PartialEq)]
pub enum PairingStep {
    /// Idle — no pairing in progress.
    Idle,
    /// Showing the sync code after initial setup.
    ShowingCode,
    /// Entering a sync code to join another device.
    Joining,
    /// Join succeeded, performing initial sync.
    Syncing,
    /// Pairing complete.
    Done,
    /// An error occurred during pairing.
    Error(String),
}
