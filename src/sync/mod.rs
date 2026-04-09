pub mod client;
pub mod credentials;
pub mod vector_clock;

#[cfg(not(test))]
pub mod http;

pub use client::{MergeResult, SyncClient, SyncOutcome};
pub use credentials::{SyncCredentials, load_clock, save_clock};
pub use vector_clock::VectorClock;

/// Trivial merge stub used until the real union-merge (#89) is implemented.
/// When clocks have diverged, we cannot safely pick a winner, so this stub
/// always reports a conflict.  This causes `SyncClient::run` to return
/// `SyncOutcome::ConflictDetected`, letting the user know their data needs
/// manual attention rather than silently discarding the server's changes.
pub fn stub_merge(local: &[u8], _server: &[u8]) -> MergeResult {
    MergeResult {
        merged: local.to_vec(),
        conflicts: vec![client::ConflictRecord {
            table: "*".into(),
            row_id: "stub-merge-placeholder".into(),
        }],
    }
}
