pub mod client;
pub mod credentials;
pub mod vector_clock;

#[cfg(not(test))]
pub mod http;

pub use client::{ConflictRecord, MergeResult, SyncClient, SyncOutcome};
pub use credentials::{SyncCredentials, delete_clock, load_clock, save_clock};
pub use vector_clock::VectorClock;

/// Trivial merge stub used until the real union-merge (#89) is implemented.
/// It returns the local blob unchanged with no conflicts.
/// This means diverged-clock cases will push the local blob back as the
/// "merged" result — a safe fallback until the full merge lands.
pub fn stub_merge(local: &[u8], _server: &[u8]) -> MergeResult {
    MergeResult {
        merged: local.to_vec(),
        conflicts: vec![],
    }
}
