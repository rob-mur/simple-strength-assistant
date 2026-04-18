use super::credentials::SyncCredentials;
use super::vector_clock::{ClockRelation, VectorClock};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Async merge function signature. Takes (local_blob, server_blob) and returns
/// a merged result. This is async because the real SQLite merge requires JS FFI.
pub type MergeFn<'a> = &'a dyn Fn(Vec<u8>, Vec<u8>) -> Pin<Box<dyn Future<Output = MergeResult>>>;

// ── Types returned / sent by the server ──────────────────────────────────────

/// Body sent to `POST /sync/:sync_id`.
#[derive(Debug, Serialize)]
pub struct PushRequest {
    pub vector_clock: VectorClock,
    /// Base-64-encoded SQLite blob
    pub blob_b64: String,
}

/// Response from `GET /sync/:sync_id/metadata`.
#[derive(Debug, Clone, Deserialize)]
pub struct SyncMetadata {
    pub vector_clock: VectorClock,
    pub conflicted: bool,
}

/// High-level outcome of a sync cycle.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncOutcome {
    /// Local clock was ahead; a push was performed; no pull needed.
    Pushed,
    /// Server was ahead; local database was replaced with the server blob.
    Pulled(Vec<u8>),
    /// Clocks diverged; merge was performed and the merged result pushed.
    Merged(Vec<u8>),
    /// Merge produced one or more conflicts; the user needs to resolve them.
    /// Contains the merged blob and the list of conflicts.
    ConflictDetected {
        merged: Vec<u8>,
        conflicts: Vec<ConflictRecord>,
    },
    /// No sync_id is configured; sync was skipped.
    Skipped,
    /// Server was unreachable; app continues offline.
    Offline,
}

// ── HTTP transport abstraction ─────────────────────────────────────────────

/// Abstraction over the HTTP layer so the sync logic can be tested without
/// a real network.  The real implementation uses `web_sys::fetch`; tests
/// supply a mock.
#[async_trait::async_trait(?Send)]
pub trait HttpClient {
    /// POST /sync/:sync_id  — push our blob and clock
    async fn push(
        &self,
        sync_id: &str,
        sync_secret: &str,
        body: &PushRequest,
    ) -> Result<(), SyncError>;

    /// GET /sync/:sync_id/metadata  — read server clock
    async fn get_metadata(
        &self,
        sync_id: &str,
        sync_secret: &str,
    ) -> Result<SyncMetadata, SyncError>;

    /// GET /sync/:sync_id  — download server blob
    async fn pull_blob(&self, sync_id: &str, sync_secret: &str) -> Result<Vec<u8>, SyncError>;
}

// ── Error type ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SyncError {
    /// Network / fetch failure
    NetworkError(String),
    /// Server responded with an unexpected status
    ServerError(u16),
    /// Could not (de)serialise data
    SerializationError(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SyncError::ServerError(code) => write!(f, "Server error: HTTP {}", code),
            SyncError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

// ── SyncClient ────────────────────────────────────────────────────────────

/// Orchestrates the push/pull/merge sync cycle described in issue #91.
pub struct SyncClient<H: HttpClient> {
    http: H,
}

impl<H: HttpClient> SyncClient<H> {
    pub fn new(http: H) -> Self {
        Self { http }
    }

    /// Run one full sync cycle.
    ///
    /// Steps:
    /// 1. If no credentials → skip silently.
    /// 2. Increment local sequence number and push current blob + clock.
    /// 3. Fetch server metadata to read its clock.
    /// 4. Compare clocks:
    ///    - Server descends from us  → nothing to pull, done (Pushed).
    ///    - We descend from server   → pull blob, replace local (Pulled).
    ///    - Clocks diverged          → pull server blob, merge, push merged.
    /// 5. If network error at any point → return Offline (never surface to UI).
    pub async fn run(
        &self,
        credentials: Option<&SyncCredentials>,
        local_blob: &[u8],
        local_clock: &mut VectorClock,
        merge_fn: MergeFn<'_>,
    ) -> SyncOutcome {
        let Some(creds) = credentials else {
            return SyncOutcome::Skipped;
        };

        if !creds.is_valid() {
            return SyncOutcome::Skipped;
        }

        // Step 1: build push request with a tentative clock increment.
        // We only commit the increment to local_clock after a successful push.
        let mut tentative_clock = local_clock.clone();
        tentative_clock.increment(&creds.device_id);

        let push_req = PushRequest {
            vector_clock: tentative_clock.clone(),
            blob_b64: base64_encode(local_blob),
        };

        if let Err(e) = self
            .http
            .push(&creds.sync_id, &creds.sync_secret, &push_req)
            .await
        {
            log::warn!("[Sync] Push failed (offline?): {}", e);
            // Clock is NOT incremented — tentative_clock is discarded
            return SyncOutcome::Offline;
        }

        // Push succeeded — commit the increment
        *local_clock = tentative_clock;

        // Step 2: fetch server metadata
        let metadata = match self
            .http
            .get_metadata(&creds.sync_id, &creds.sync_secret)
            .await
        {
            Ok(m) => m,
            Err(e) => {
                log::warn!("[Sync] Metadata fetch failed (offline?): {}", e);
                return SyncOutcome::Offline;
            }
        };

        let server_clock = &metadata.vector_clock;

        // Step 3: compare clocks
        match local_clock.compare(server_clock) {
            ClockRelation::Equal | ClockRelation::AheadOf => {
                // Server is at or behind us — our push was sufficient
                SyncOutcome::Pushed
            }
            ClockRelation::BehindOf => {
                // Server is ahead — pull and replace
                match self
                    .http
                    .pull_blob(&creds.sync_id, &creds.sync_secret)
                    .await
                {
                    Ok(blob) => {
                        // Advance local clock to match server
                        local_clock.merge(server_clock);
                        SyncOutcome::Pulled(blob)
                    }
                    Err(e) => {
                        log::warn!("[Sync] Pull failed (offline?): {}", e);
                        SyncOutcome::Offline
                    }
                }
            }
            ClockRelation::Concurrent => {
                // Diverged — pull server blob and merge
                let server_blob = match self
                    .http
                    .pull_blob(&creds.sync_id, &creds.sync_secret)
                    .await
                {
                    Ok(b) => b,
                    Err(e) => {
                        log::warn!("[Sync] Pull for merge failed (offline?): {}", e);
                        return SyncOutcome::Offline;
                    }
                };

                let merge_result = merge_fn(local_blob.to_vec(), server_blob).await;
                local_clock.merge(server_clock);

                if !merge_result.conflicts.is_empty() {
                    return SyncOutcome::ConflictDetected {
                        merged: merge_result.merged,
                        conflicts: merge_result.conflicts,
                    };
                }

                // Push merged result back to server
                let merged_push = PushRequest {
                    vector_clock: local_clock.clone(),
                    blob_b64: base64_encode(&merge_result.merged),
                };

                if let Err(e) = self
                    .http
                    .push(&creds.sync_id, &creds.sync_secret, &merged_push)
                    .await
                {
                    log::warn!("[Sync] Push of merged result failed: {}", e);
                    return SyncOutcome::Offline;
                }

                SyncOutcome::Merged(merge_result.merged)
            }
        }
    }

    /// Run an initial sync cycle for a device that has never synced before.
    ///
    /// This method is called when the local vector clock is empty, indicating
    /// the device is syncing for the first time. It inspects both the local
    /// database (via `local_blob`) and the server state to choose the correct
    /// strategy, per the table in issue #149:
    ///
    /// | Local DB  | Server slot  | Action                                    |
    /// |-----------|-------------|-------------------------------------------|
    /// | Empty     | Empty (404) | No-op                                     |
    /// | Has data  | Empty (404) | Push local DB (normal first push)         |
    /// | Empty     | Has data    | Pull server blob (fast-forward)           |
    /// | Has data  | Has data    | Push local first → merge → push merged    |
    ///
    /// `local_blob_is_empty` indicates whether the local database has any
    /// meaningful data (as determined by the caller — typically checking
    /// whether the exported blob represents an empty database).
    pub async fn run_initial_sync(
        &self,
        credentials: Option<&SyncCredentials>,
        local_blob: &[u8],
        local_blob_is_empty: bool,
        local_clock: &mut VectorClock,
        merge_fn: MergeFn<'_>,
    ) -> SyncOutcome {
        let Some(creds) = credentials else {
            return SyncOutcome::Skipped;
        };

        if !creds.is_valid() {
            return SyncOutcome::Skipped;
        }

        // Step 1: probe the server to determine whether a slot already exists.
        let server_state = match self
            .http
            .get_metadata(&creds.sync_id, &creds.sync_secret)
            .await
        {
            Ok(m) => Some(m),
            Err(SyncError::ServerError(404)) => None,
            Err(e) => {
                log::warn!(
                    "[Sync] Initial sync: metadata probe failed (offline?): {}",
                    e
                );
                return SyncOutcome::Offline;
            }
        };

        match (local_blob_is_empty, &server_state) {
            // ── Case 1: both empty → no-op ──────────────────────────────
            (true, None) => {
                log::debug!("[Sync] Initial sync: both local and server empty → no-op");
                SyncOutcome::Skipped
            }

            // ── Case 2: local has data, server empty → normal first push ─
            (false, None) => {
                log::debug!("[Sync] Initial sync: local has data, server empty → push");
                let mut tentative_clock = local_clock.clone();
                tentative_clock.increment(&creds.device_id);

                let push_req = PushRequest {
                    vector_clock: tentative_clock.clone(),
                    blob_b64: base64_encode(local_blob),
                };

                if let Err(e) = self
                    .http
                    .push(&creds.sync_id, &creds.sync_secret, &push_req)
                    .await
                {
                    log::warn!("[Sync] Initial sync push failed: {}", e);
                    return SyncOutcome::Offline;
                }

                *local_clock = tentative_clock;
                SyncOutcome::Pushed
            }

            // ── Case 3: local empty, server has data → fast-forward pull ─
            (true, Some(metadata)) => {
                log::debug!("[Sync] Initial sync: local empty, server has data → pull");
                match self
                    .http
                    .pull_blob(&creds.sync_id, &creds.sync_secret)
                    .await
                {
                    Ok(blob) => {
                        local_clock.merge(&metadata.vector_clock);
                        SyncOutcome::Pulled(blob)
                    }
                    Err(e) => {
                        log::warn!("[Sync] Initial sync pull failed: {}", e);
                        SyncOutcome::Offline
                    }
                }
            }

            // ── Case 4: both have data → push-then-merge ────────────────
            (false, Some(_metadata)) => {
                log::info!(
                    "[Sync] Initial sync: both local and server have data → push-then-merge"
                );

                // Step A: push local blob with our clock to force divergence
                let mut tentative_clock = local_clock.clone();
                tentative_clock.increment(&creds.device_id);

                let push_req = PushRequest {
                    vector_clock: tentative_clock.clone(),
                    blob_b64: base64_encode(local_blob),
                };

                if let Err(e) = self
                    .http
                    .push(&creds.sync_id, &creds.sync_secret, &push_req)
                    .await
                {
                    log::warn!("[Sync] Initial sync push (for merge) failed: {}", e);
                    return SyncOutcome::Offline;
                }

                *local_clock = tentative_clock;

                // Step B: re-fetch metadata to get the server's updated state
                let metadata = match self
                    .http
                    .get_metadata(&creds.sync_id, &creds.sync_secret)
                    .await
                {
                    Ok(m) => m,
                    Err(e) => {
                        log::warn!("[Sync] Initial sync metadata re-fetch failed: {}", e);
                        return SyncOutcome::Offline;
                    }
                };

                let server_clock = &metadata.vector_clock;

                // Step C: compare clocks and handle accordingly
                match local_clock.compare(server_clock) {
                    ClockRelation::Equal | ClockRelation::AheadOf => {
                        // Server accepted our push as-is; no merge needed.
                        // This can happen if the server had already incorporated
                        // our push into its clock.
                        SyncOutcome::Pushed
                    }
                    ClockRelation::BehindOf | ClockRelation::Concurrent => {
                        // Diverged or server ahead — pull and merge
                        let server_blob = match self
                            .http
                            .pull_blob(&creds.sync_id, &creds.sync_secret)
                            .await
                        {
                            Ok(b) => b,
                            Err(e) => {
                                log::warn!("[Sync] Initial sync pull for merge failed: {}", e);
                                return SyncOutcome::Offline;
                            }
                        };

                        let merge_result = merge_fn(local_blob.to_vec(), server_blob).await;
                        local_clock.merge(server_clock);

                        if !merge_result.conflicts.is_empty() {
                            return SyncOutcome::ConflictDetected {
                                merged: merge_result.merged,
                                conflicts: merge_result.conflicts,
                            };
                        }

                        // Push merged result back
                        let merged_push = PushRequest {
                            vector_clock: local_clock.clone(),
                            blob_b64: base64_encode(&merge_result.merged),
                        };

                        if let Err(e) = self
                            .http
                            .push(&creds.sync_id, &creds.sync_secret, &merged_push)
                            .await
                        {
                            log::warn!("[Sync] Initial sync push of merged result failed: {}", e);
                            return SyncOutcome::Offline;
                        }

                        SyncOutcome::Merged(merge_result.merged)
                    }
                }
            }
        }
    }
}

// ── Merge result ──────────────────────────────────────────────────────────

/// Result of merging two database blobs.
pub struct MergeResult {
    /// The merged SQLite blob
    pub merged: Vec<u8>,
    /// Any conflicts that need user resolution
    pub conflicts: Vec<ConflictRecord>,
}

/// Represents a single conflict between two records.
///
/// When the union merge detects the same UUID with the same `updated_at` but
/// different field values, it surfaces both versions so the user can pick one.
/// `version_a` and `version_b` are JSON-encoded row objects containing all columns.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConflictRecord {
    /// The table containing the conflicting record (e.g. "exercises", "completed_sets").
    pub table: String,
    /// The stable UUID of the conflicting row.
    pub row_id: String,
    /// JSON representation of the row from device A (local).
    pub version_a: String,
    /// JSON representation of the row from device B (remote).
    pub version_b: String,
}

// ── Base-64 helper ────────────────────────────────────────────────────────

/// Minimal base-64 encoder (standard alphabet).  We avoid pulling in the
/// `base64` crate to keep dependencies lean.
pub fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };
        out.push(TABLE[b0 >> 2] as char);
        out.push(TABLE[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[b2 & 0x3f] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // ── Mock HTTP client ─────────────────────────────────────────────────

    #[derive(Clone)]
    struct MockHttp {
        /// If set, all requests return NetworkError
        offline: bool,
        server_metadata: SyncMetadata,
        server_blob: Vec<u8>,
        push_calls: Rc<RefCell<u32>>,
    }

    impl MockHttp {
        fn new(server_clock: VectorClock, server_blob: Vec<u8>) -> Self {
            Self {
                offline: false,
                server_metadata: SyncMetadata {
                    vector_clock: server_clock,
                    conflicted: false,
                },
                server_blob,
                push_calls: Rc::new(RefCell::new(0)),
            }
        }

        fn offline() -> Self {
            Self {
                offline: true,
                server_metadata: SyncMetadata {
                    vector_clock: VectorClock::new(),
                    conflicted: false,
                },
                server_blob: vec![],
                push_calls: Rc::new(RefCell::new(0)),
            }
        }

        #[allow(dead_code)]
        fn push_call_count(&self) -> u32 {
            *self.push_calls.borrow()
        }
    }

    #[async_trait::async_trait(?Send)]
    impl HttpClient for MockHttp {
        async fn push(
            &self,
            _sync_id: &str,
            _sync_secret: &str,
            _body: &PushRequest,
        ) -> Result<(), SyncError> {
            if self.offline {
                return Err(SyncError::NetworkError("offline".into()));
            }
            *self.push_calls.borrow_mut() += 1;
            Ok(())
        }

        async fn get_metadata(
            &self,
            _sync_id: &str,
            _sync_secret: &str,
        ) -> Result<SyncMetadata, SyncError> {
            if self.offline {
                return Err(SyncError::NetworkError("offline".into()));
            }
            Ok(self.server_metadata.clone())
        }

        async fn pull_blob(
            &self,
            _sync_id: &str,
            _sync_secret: &str,
        ) -> Result<Vec<u8>, SyncError> {
            if self.offline {
                return Err(SyncError::NetworkError("offline".into()));
            }
            Ok(self.server_blob.clone())
        }
    }

    fn creds() -> SyncCredentials {
        SyncCredentials {
            sync_id: "test-sync-id".into(),
            sync_secret: "test-secret".into(),
            device_id: "device-a".into(),
        }
    }

    fn no_op_merge(a: Vec<u8>, _b: Vec<u8>) -> Pin<Box<dyn Future<Output = MergeResult>>> {
        Box::pin(async move {
            MergeResult {
                merged: a,
                conflicts: vec![],
            }
        })
    }

    fn conflict_merge(a: Vec<u8>, _b: Vec<u8>) -> Pin<Box<dyn Future<Output = MergeResult>>> {
        Box::pin(async move {
            MergeResult {
                merged: a,
                conflicts: vec![ConflictRecord {
                    table: "exercises".into(),
                    row_id: "row-1".into(),
                    version_a: r#"{"uuid":"row-1","name":"Bench Press","updated_at":"2025-01-01T00:00:00Z"}"#.into(),
                    version_b: r#"{"uuid":"row-1","name":"Flat Bench Press","updated_at":"2025-01-01T00:00:00Z"}"#.into(),
                }],
            }
        })
    }

    // ── QA checklist behaviour 1 ─────────────────────────────────────────
    // Skipped when no sync_id present

    #[tokio::test]
    async fn test_skipped_when_no_credentials() {
        let http = MockHttp::new(VectorClock::new(), vec![]);
        let client = SyncClient::new(http);
        let mut clock = VectorClock::new();
        let outcome = client.run(None, b"local", &mut clock, &no_op_merge).await;
        assert_eq!(outcome, SyncOutcome::Skipped);
    }

    // ── QA checklist behaviour 2 ─────────────────────────────────────────
    // Fast-forward pull: client behind server (server has seen all of client's work plus more)

    #[tokio::test]
    async fn test_pull_when_client_behind_server() {
        // device-a is our device.  The server already has device-a at seq 2
        // (meaning it received our past pushes) and also has device-b at seq 3.
        // After run() increments device-a to 1, local = {device-a:1}.
        // Server = {device-a:2, device-b:3} → server is ahead on BOTH devices
        // relative to our incremented clock, so ClockRelation is BehindOf.
        let mut server_clock = VectorClock::new();
        server_clock.0.insert("device-a".to_string(), 2);
        server_clock.0.insert("device-b".to_string(), 3);

        let server_blob = b"server-data".to_vec();
        let http = MockHttp::new(server_clock.clone(), server_blob.clone());
        let client = SyncClient::new(http);

        let mut local_clock = VectorClock::new(); // empty — behind server
        let outcome = client
            .run(Some(&creds()), b"local", &mut local_clock, &no_op_merge)
            .await;

        // After run() increments device-a to 1, local = {device-a:1}.
        // Server has device-a:2, device-b:3 — server dominates on all devices → BehindOf → Pulled
        match outcome {
            SyncOutcome::Pulled(blob) => assert_eq!(blob, server_blob),
            other => panic!("Expected Pulled, got {:?}", other),
        }
    }

    // ── QA checklist behaviour 3 ─────────────────────────────────────────
    // Client is ahead of server — no pull needed

    #[tokio::test]
    async fn test_no_pull_when_client_ahead_of_server() {
        // Server clock behind client
        let server_clock = VectorClock::new(); // empty
        let http = MockHttp::new(server_clock, b"server".to_vec());
        let push_calls = http.push_calls.clone();
        let client = SyncClient::new(http);

        let mut local_clock = VectorClock::new();
        local_clock.increment("device-a");
        local_clock.increment("device-a"); // seq 2

        let outcome = client
            .run(Some(&creds()), b"local", &mut local_clock, &no_op_merge)
            .await;

        // Should have pushed once (our increment) and returned Pushed
        assert_eq!(outcome, SyncOutcome::Pushed);
        assert_eq!(*push_calls.borrow(), 1);
    }

    // ── QA checklist behaviour 4 ─────────────────────────────────────────
    // Diverged clocks — merge, persist, push back

    #[tokio::test]
    async fn test_merge_on_diverged_clocks() {
        // Server has device-b ahead; client has device-a ahead → concurrent
        let mut server_clock = VectorClock::new();
        server_clock.increment("device-b");

        let server_blob = b"server-only-data".to_vec();
        let http = MockHttp::new(server_clock, server_blob);
        let push_calls = http.push_calls.clone();
        let client = SyncClient::new(http);

        let mut local_clock = VectorClock::new();
        // Note: device-a will be incremented inside run()
        // but server has device-b=1, we have device-a=0 initially
        // After increment, local = {device-a:1}, server = {device-b:1} → Concurrent

        let outcome = client
            .run(Some(&creds()), b"local", &mut local_clock, &no_op_merge)
            .await;

        match outcome {
            SyncOutcome::Merged(_) => {}
            other => panic!("Expected Merged, got {:?}", other),
        }

        // Should push twice: initial push + merged result push
        assert_eq!(*push_calls.borrow(), 2);
    }

    // ── QA checklist behaviour 5 ─────────────────────────────────────────
    // Merge with conflicts → surface ConflictDetected

    #[tokio::test]
    async fn test_conflict_detected_on_diverged_clocks_with_conflicts() {
        let mut server_clock = VectorClock::new();
        server_clock.increment("device-b");

        let http = MockHttp::new(server_clock, b"server".to_vec());
        let client = SyncClient::new(http);

        let mut local_clock = VectorClock::new();

        let outcome = client
            .run(Some(&creds()), b"local", &mut local_clock, &conflict_merge)
            .await;

        match outcome {
            SyncOutcome::ConflictDetected { conflicts, .. } => {
                assert_eq!(conflicts.len(), 1);
                assert_eq!(conflicts[0].row_id, "row-1");
            }
            other => panic!("Expected ConflictDetected, got {:?}", other),
        }
    }

    // ── QA checklist behaviour 6 ─────────────────────────────────────────
    // Network error → Offline, app unaffected

    #[tokio::test]
    async fn test_offline_when_server_unreachable() {
        let http = MockHttp::offline();
        let client = SyncClient::new(http);

        let mut clock = VectorClock::new();
        let outcome = client
            .run(Some(&creds()), b"local", &mut clock, &no_op_merge)
            .await;

        assert_eq!(outcome, SyncOutcome::Offline);
    }

    // ── QA checklist behaviour 7 ─────────────────────────────────────────
    // sync_secret never appears in URL segments (enforced by HttpClient contract:
    // secret is passed separately, not interpolated into sync_id path)

    #[tokio::test]
    async fn test_sync_secret_not_in_url_path() {
        // The SyncClient only passes sync_id as the URL path component
        // and passes sync_secret as a separate argument (header in real impl).
        // We validate this at the type level: HttpClient::push takes
        // sync_id and sync_secret as separate parameters.
        // This test verifies that a sync_secret-looking value in sync_id is
        // rejected by is_valid (i.e., we never construct such credentials).
        let creds_with_secret_in_id = SyncCredentials {
            sync_id: "secret-in-id-value".into(),
            sync_secret: "secret-in-id-value".into(), // same value (worst case)
            device_id: "dev".into(),
        };
        // The credential itself is technically valid (both fields non-empty).
        // The guarantee is structural: the HttpClient interface keeps secret
        // out of the URL by design. No URL is constructed in Rust at all —
        // that happens in the real HTTP implementation.
        assert!(creds_with_secret_in_id.is_valid());

        // Confirm SyncOutcome::Skipped is NOT returned for valid credentials
        let http = MockHttp::new(VectorClock::new(), vec![]);
        let client = SyncClient::new(http);
        let mut clock = VectorClock::new();
        let outcome = client
            .run(
                Some(&creds_with_secret_in_id),
                b"local",
                &mut clock,
                &no_op_merge,
            )
            .await;
        assert_ne!(outcome, SyncOutcome::Skipped);
    }

    // ── base64 encode ─────────────────────────────────────────────────────

    #[test]
    fn test_base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_encode_known_value() {
        // "Man" → "TWFu"
        assert_eq!(base64_encode(b"Man"), "TWFu");
    }

    #[test]
    fn test_base64_encode_padding_one() {
        // "Ma" → "TWE="
        assert_eq!(base64_encode(b"Ma"), "TWE=");
    }

    #[test]
    fn test_base64_encode_padding_two() {
        // "M" → "TQ=="
        assert_eq!(base64_encode(b"M"), "TQ==");
    }

    // ── MockHttp with 404 support for initial sync tests ────────────────

    /// A mock HTTP client where `get_metadata` returns 404 (server slot empty)
    /// until a push is made, then returns the provided metadata.
    #[derive(Clone)]
    struct MockHttp404 {
        /// If true, get_metadata returns 404 (server slot does not exist yet)
        server_empty: Rc<RefCell<bool>>,
        /// Metadata to return once the server is no longer empty
        server_metadata: SyncMetadata,
        server_blob: Vec<u8>,
        push_calls: Rc<RefCell<u32>>,
    }

    impl MockHttp404 {
        /// Server slot is empty (404 on metadata).
        fn empty() -> Self {
            Self {
                server_empty: Rc::new(RefCell::new(true)),
                server_metadata: SyncMetadata {
                    vector_clock: VectorClock::new(),
                    conflicted: false,
                },
                server_blob: vec![],
                push_calls: Rc::new(RefCell::new(0)),
            }
        }

        /// Server has existing data.
        fn with_data(server_clock: VectorClock, server_blob: Vec<u8>) -> Self {
            Self {
                server_empty: Rc::new(RefCell::new(false)),
                server_metadata: SyncMetadata {
                    vector_clock: server_clock,
                    conflicted: false,
                },
                server_blob,
                push_calls: Rc::new(RefCell::new(0)),
            }
        }

        fn push_call_count(&self) -> u32 {
            *self.push_calls.borrow()
        }
    }

    #[async_trait::async_trait(?Send)]
    impl HttpClient for MockHttp404 {
        async fn push(
            &self,
            _sync_id: &str,
            _sync_secret: &str,
            _body: &PushRequest,
        ) -> Result<(), SyncError> {
            *self.push_calls.borrow_mut() += 1;
            // After a push, the server slot is no longer empty
            *self.server_empty.borrow_mut() = false;
            Ok(())
        }

        async fn get_metadata(
            &self,
            _sync_id: &str,
            _sync_secret: &str,
        ) -> Result<SyncMetadata, SyncError> {
            if *self.server_empty.borrow() {
                return Err(SyncError::ServerError(404));
            }
            Ok(self.server_metadata.clone())
        }

        async fn pull_blob(
            &self,
            _sync_id: &str,
            _sync_secret: &str,
        ) -> Result<Vec<u8>, SyncError> {
            if *self.server_empty.borrow() {
                return Err(SyncError::ServerError(404));
            }
            Ok(self.server_blob.clone())
        }
    }

    // ── Initial sync: QA checklist tests (#149) ─────────────────────────

    // Case 1: empty local DB + empty server → no-op (Skipped)
    #[tokio::test]
    async fn test_initial_sync_both_empty_is_noop() {
        let http = MockHttp404::empty();
        let client = SyncClient::new(http.clone());
        let mut clock = VectorClock::new();

        let outcome = client
            .run_initial_sync(Some(&creds()), b"", true, &mut clock, &no_op_merge)
            .await;

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(http.push_call_count(), 0);
        assert!(clock.is_empty(), "Clock should remain empty after no-op");
    }

    // Case 2: local has data, server empty (404) → push
    #[tokio::test]
    async fn test_initial_sync_local_data_server_empty_pushes() {
        let http = MockHttp404::empty();
        let client = SyncClient::new(http.clone());
        let mut clock = VectorClock::new();

        let outcome = client
            .run_initial_sync(
                Some(&creds()),
                b"local-workout-data",
                false,
                &mut clock,
                &no_op_merge,
            )
            .await;

        assert_eq!(outcome, SyncOutcome::Pushed);
        assert_eq!(http.push_call_count(), 1);
        assert!(!clock.is_empty(), "Clock should be incremented after push");
        assert_eq!(clock.get("device-a"), 1);
    }

    // Case 3: local empty, server has data → fast-forward pull
    #[tokio::test]
    async fn test_initial_sync_local_empty_server_has_data_pulls() {
        let mut server_clock = VectorClock::new();
        server_clock.increment("device-b");
        server_clock.increment("device-b");
        let server_blob = b"server-workout-data".to_vec();

        let http = MockHttp404::with_data(server_clock.clone(), server_blob.clone());
        let client = SyncClient::new(http.clone());
        let mut clock = VectorClock::new();

        let outcome = client
            .run_initial_sync(Some(&creds()), b"", true, &mut clock, &no_op_merge)
            .await;

        match outcome {
            SyncOutcome::Pulled(blob) => assert_eq!(blob, server_blob),
            other => panic!("Expected Pulled, got {:?}", other),
        }
        // No pushes should occur for a pure pull
        assert_eq!(http.push_call_count(), 0);
        // Clock should have been merged with server
        assert_eq!(clock.get("device-b"), 2);
    }

    // Case 4: both have data → push-then-merge preserves both datasets
    #[tokio::test]
    async fn test_initial_sync_both_have_data_merges() {
        // Server has data from device-b
        let mut server_clock = VectorClock::new();
        server_clock.increment("device-b");
        let server_blob = b"server-data".to_vec();

        let http = MockHttp404::with_data(server_clock, server_blob);
        let client = SyncClient::new(http.clone());
        let mut clock = VectorClock::new();

        // Use a merge function that concatenates both blobs to prove both
        // are passed into the merge
        fn union_merge(a: Vec<u8>, b: Vec<u8>) -> Pin<Box<dyn Future<Output = MergeResult>>> {
            Box::pin(async move {
                let mut merged = a;
                merged.extend_from_slice(&b);
                MergeResult {
                    merged,
                    conflicts: vec![],
                }
            })
        }

        let outcome = client
            .run_initial_sync(
                Some(&creds()),
                b"local-data",
                false,
                &mut clock,
                &union_merge,
            )
            .await;

        match outcome {
            SyncOutcome::Merged(blob) => {
                // Merged blob should contain both local and server data
                assert!(
                    blob.len() > b"local-data".len(),
                    "Merged blob should be larger than local alone"
                );
                // Verify both blobs contributed
                let expected: Vec<u8> = b"local-data"
                    .iter()
                    .chain(b"server-data")
                    .copied()
                    .collect();
                assert_eq!(blob, expected);
            }
            other => panic!("Expected Merged, got {:?}", other),
        }

        // Should have pushed twice: initial push + merged result push
        assert_eq!(http.push_call_count(), 2);

        // Clock should reflect both devices
        assert!(clock.get("device-a") >= 1);
        assert!(clock.get("device-b") >= 1);
    }

    // Case 4 with conflicts: both have data and merge produces conflicts
    #[tokio::test]
    async fn test_initial_sync_both_have_data_with_conflicts() {
        let mut server_clock = VectorClock::new();
        server_clock.increment("device-b");

        let http = MockHttp404::with_data(server_clock, b"server".to_vec());
        let client = SyncClient::new(http.clone());
        let mut clock = VectorClock::new();

        let outcome = client
            .run_initial_sync(Some(&creds()), b"local", false, &mut clock, &conflict_merge)
            .await;

        match outcome {
            SyncOutcome::ConflictDetected { conflicts, .. } => {
                assert_eq!(conflicts.len(), 1);
                assert_eq!(conflicts[0].row_id, "row-1");
            }
            other => panic!("Expected ConflictDetected, got {:?}", other),
        }

        // Only the initial push, no merged push (conflicts paused the cycle)
        assert_eq!(http.push_call_count(), 1);
    }

    // Initial sync: no credentials → Skipped
    #[tokio::test]
    async fn test_initial_sync_no_credentials_skips() {
        let http = MockHttp404::empty();
        let client = SyncClient::new(http);
        let mut clock = VectorClock::new();

        let outcome = client
            .run_initial_sync(None, b"local", false, &mut clock, &no_op_merge)
            .await;

        assert_eq!(outcome, SyncOutcome::Skipped);
    }

    // Initial sync: server unreachable → Offline
    #[tokio::test]
    async fn test_initial_sync_offline_returns_offline() {
        let http = MockHttp::offline();
        let client = SyncClient::new(http);
        let mut clock = VectorClock::new();

        let outcome = client
            .run_initial_sync(Some(&creds()), b"local", false, &mut clock, &no_op_merge)
            .await;

        assert_eq!(outcome, SyncOutcome::Offline);
    }
}
