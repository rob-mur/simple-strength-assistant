pub mod client;
pub mod credentials;
pub mod vector_clock;

#[cfg(not(test))]
pub mod http;

pub use client::{ConflictRecord, MergeResult, SyncClient, SyncOutcome};
pub use credentials::{SyncCredentials, delete_clock, load_clock, save_clock};
pub use vector_clock::VectorClock;

/// Trivial merge stub for tests. Returns the local blob unchanged.
#[cfg(test)]
pub fn stub_merge(
    local: Vec<u8>,
    _server: Vec<u8>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = MergeResult>>> {
    Box::pin(async move {
        MergeResult {
            merged: local,
            conflicts: vec![],
        }
    })
}
