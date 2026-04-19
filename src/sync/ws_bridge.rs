/// WebSocket-based CRR changeset sync bridge.
///
/// This module provides the Rust-to-JS FFI layer for the new vlcn.io/crsqlite
/// WebSocket sync protocol.  It imports functions from `sync-module.js` via
/// `wasm_bindgen` and exposes a high-level `run_ws_sync()` function that
/// `trigger_background_sync()` calls.
///
/// The JS module handles:
///   - Opening a WebSocket connection to the sync server
///   - Extracting local changesets via `crsql_changes()`
///   - Sending changesets and applying received ones
///   - Closing the connection after the exchange
///
/// This Rust module handles:
///   - Calling the JS sync function with credentials
///   - Interpreting the result string into a `WsSyncOutcome`
///   - Logging and error handling

// ── JS FFI bindings (WASM-only) ─────────────────────────────────────────────

#[cfg(not(test))]
mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/public/sync-module.js")]
    extern "C" {
        /// Run one WebSocket sync cycle.
        ///
        /// Returns a promise that resolves to a string:
        ///   "synced"       — changes were exchanged successfully
        ///   "no_changes"   — connected but nothing to sync
        ///   "offline"      — could not connect to the server
        ///   "error:<msg>"  — an error occurred
        #[wasm_bindgen(js_name = runSyncCycle)]
        pub async fn run_sync_cycle_js(
            sync_id: &str,
            sync_secret: &str,
            timeout_ms: u32,
        ) -> JsValue;

        /// Check whether the sync server is reachable.
        #[wasm_bindgen(js_name = checkSyncServerHealth)]
        pub async fn check_sync_server_health_js(sync_id: &str) -> JsValue;
    }
}

// ── Outcome type ─────────────────────────────────────────────────────────────

/// Result of a single WebSocket sync cycle.
#[derive(Debug, Clone, PartialEq)]
pub enum WsSyncOutcome {
    /// Changes were exchanged with the server.
    Synced,
    /// Connected successfully but there were no changes to exchange.
    NoChanges,
    /// Server was unreachable (network error, timeout, etc.).
    Offline,
    /// An error occurred during the sync cycle.
    Error(String),
}

// ── Public API (WASM-only) ───────────────────────────────────────────────────

/// Default timeout for a single sync cycle (15 seconds).
#[cfg(not(test))]
const SYNC_TIMEOUT_MS: u32 = 15_000;

/// Run one WebSocket-based CRR changeset sync cycle.
///
/// This is the main entry point called by `trigger_background_sync()`.
/// It delegates to the JS sync module which manages the WebSocket connection
/// and changeset exchange.
#[cfg(not(test))]
pub async fn run_ws_sync(sync_id: &str, sync_secret: &str) -> WsSyncOutcome {
    log::debug!("[WS Sync] Starting sync cycle for slot {}", sync_id);

    let result = ffi::run_sync_cycle_js(sync_id, sync_secret, SYNC_TIMEOUT_MS).await;

    let outcome_str = result
        .as_string()
        .unwrap_or_else(|| "error:unknown".to_string());

    let outcome = parse_outcome(&outcome_str);

    match &outcome {
        WsSyncOutcome::Synced => log::info!("[WS Sync] Sync completed — changes exchanged"),
        WsSyncOutcome::NoChanges => log::debug!("[WS Sync] Sync completed — no changes"),
        WsSyncOutcome::Offline => log::warn!("[WS Sync] Server unreachable"),
        WsSyncOutcome::Error(msg) => log::warn!("[WS Sync] Error: {}", msg),
    }

    outcome
}

/// Check whether the sync server is reachable.
#[cfg(not(test))]
pub async fn is_server_reachable(sync_id: &str) -> bool {
    let result = ffi::check_sync_server_health_js(sync_id).await;
    result.as_bool().unwrap_or(false)
}

// ── Pure logic (available in tests) ──────────────────────────────────────────

/// Parse the outcome string returned by the JS sync module.
pub fn parse_outcome(s: &str) -> WsSyncOutcome {
    match s {
        "synced" => WsSyncOutcome::Synced,
        "no_changes" => WsSyncOutcome::NoChanges,
        "offline" => WsSyncOutcome::Offline,
        other if other.starts_with("error:") => WsSyncOutcome::Error(other[6..].to_string()),
        other => WsSyncOutcome::Error(format!("unexpected outcome: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_outcome_synced() {
        assert_eq!(parse_outcome("synced"), WsSyncOutcome::Synced);
    }

    #[test]
    fn test_parse_outcome_no_changes() {
        assert_eq!(parse_outcome("no_changes"), WsSyncOutcome::NoChanges);
    }

    #[test]
    fn test_parse_outcome_offline() {
        assert_eq!(parse_outcome("offline"), WsSyncOutcome::Offline);
    }

    #[test]
    fn test_parse_outcome_error() {
        assert_eq!(
            parse_outcome("error:connection refused"),
            WsSyncOutcome::Error("connection refused".to_string())
        );
    }

    #[test]
    fn test_parse_outcome_unknown() {
        assert_eq!(
            parse_outcome("garbage"),
            WsSyncOutcome::Error("unexpected outcome: garbage".to_string())
        );
    }
}
