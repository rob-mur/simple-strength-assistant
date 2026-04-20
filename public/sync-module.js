// WebSocket-based CRR changeset sync module.
//
// Uses the crsql_changes() virtual table provided by crsqlite-wasm to extract
// and apply changesets.  Communicates with the sync server over a WebSocket
// connection (one per sync cycle).
//
// This module is called from Rust via wasm_bindgen FFI (see src/sync/ws_bridge.rs).
// Sync results are communicated back to Rust through resolved promises.

// Lazily-resolved reference to the live crsqlite DB handle.
// The db-module.js initDatabase() stores the open handle on this module-level
// variable via registerSyncDb().
let _db = null;

/**
 * Register the live crsqlite database handle for use by the sync module.
 * Called by db-module.js after the database is opened.
 *
 * @param {object} db  The crsqlite-wasm DB instance.
 */
export function registerSyncDb(db) {
  _db = db;
}

function getDb() {
  if (!_db) {
    throw new Error("[Sync] Database not registered with sync module. Call registerSyncDb() first.");
  }
  return _db;
}

/**
 * Get the current db_version (site's latest change version).
 * Used to determine which changes to send.
 *
 * @returns {Promise<bigint>} The current db_version.
 */
async function getDbVersion() {
  const db = getDb();
  const rows = await db.execA("SELECT crsql_db_version()");
  if (rows && rows.length > 0) {
    return BigInt(rows[0][0]);
  }
  return 0n;
}

/**
 * Get the site_id for this database instance (a unique 16-byte identifier).
 *
 * @returns {Promise<Uint8Array>} The site_id bytes.
 */
async function getSiteId() {
  const db = getDb();
  const rows = await db.execA("SELECT crsql_site_id()");
  if (rows && rows.length > 0) {
    return rows[0][0];
  }
  throw new Error("[Sync] Could not read crsql_site_id()");
}

/**
 * Extract local changesets since the given version.
 *
 * The crsql_changes() virtual table returns rows with columns:
 *   [table, pk, cid, val, col_version, db_version, site_id, cl, seq]
 *
 * @param {bigint} sinceVersion  Only return changes with db_version > sinceVersion.
 * @returns {Promise<Array>} Array of change rows.
 */
async function getChangesSince(sinceVersion) {
  const db = getDb();
  const rows = await db.execA(
    "SELECT [table], [pk], [cid], [val], [col_version], [db_version], [site_id], [cl], [seq] " +
    "FROM crsql_changes WHERE db_version > ?",
    [sinceVersion]
  );
  return rows || [];
}

/**
 * Apply remote changesets received from the server.
 *
 * Each change is inserted into crsql_changes() which handles CRDT merge logic.
 *
 * @param {Array<Array>} changes  Array of change rows (same column order as getChangesSince).
 */
async function applyChanges(changes) {
  if (!changes || changes.length === 0) return;

  const db = getDb();
  await db.exec("BEGIN");
  try {
    for (const change of changes) {
      await db.exec(
        "INSERT INTO crsql_changes ([table], [pk], [cid], [val], [col_version], [db_version], [site_id], [cl], [seq]) " +
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        change
      );
    }
    await db.exec("COMMIT");
  } catch (e) {
    await db.exec("ROLLBACK");
    throw e;
  }
}

/**
 * Encode changesets as JSON for WebSocket transmission.
 * BigInt values are converted to strings for JSON compatibility.
 *
 * @param {Array<Array>} changes  Change rows from getChangesSince.
 * @returns {string} JSON-encoded changes.
 */
function encodeChanges(changes) {
  return JSON.stringify(changes, (_key, value) =>
    typeof value === "bigint" ? value.toString() : value
  );
}

/**
 * Build the WebSocket URL from the HTTP sync base URL and sync_id.
 *
 * @param {string} syncId  The sync slot identifier.
 * @returns {string} WebSocket URL.
 */
function buildWsUrl(syncId) {
  // Read the sync base URL (injected at build time).
  let base = window.SYNC_BASE_URL || "";
  if (!base || base.includes("%%")) {
    // Fallback: derive from current page origin.
    base = window.location.origin + "/api";
  }

  // Convert http(s) to ws(s).
  const wsBase = base.replace(/^http/, "ws");
  return `${wsBase.replace(/\/$/, "")}/sync/${syncId}/ws`;
}

// ── Sync-state tracking ───────────────────────────────────────────────────────
// The server tracks the last db_version it has seen from each client.  On the
// client side we persist the "last version we sent" in localStorage so we only
// send new changesets on each sync cycle.

const LAST_SENT_KEY = "sync_last_sent_version";

function getLastSentVersion() {
  try {
    const raw = localStorage.getItem(LAST_SENT_KEY);
    return raw ? BigInt(raw) : 0n;
  } catch {
    return 0n;
  }
}

function setLastSentVersion(version) {
  try {
    localStorage.setItem(LAST_SENT_KEY, version.toString());
  } catch {
    // localStorage may be unavailable in some contexts; ignore.
  }
}

// Track the last version we received from the server so we can tell the
// server which changes to send us.
const LAST_RECEIVED_KEY = "sync_last_received_version";

function getLastReceivedVersion() {
  try {
    const raw = localStorage.getItem(LAST_RECEIVED_KEY);
    return raw ? BigInt(raw) : 0n;
  } catch {
    return 0n;
  }
}

function setLastReceivedVersion(version) {
  try {
    localStorage.setItem(LAST_RECEIVED_KEY, version.toString());
  } catch {
    // Ignore.
  }
}

/**
 * Run one sync cycle over WebSocket.
 *
 * Protocol:
 *   1. Client opens a WebSocket to /sync/:sync_id/ws
 *   2. Client sends: { type: "push", site_id, changes, last_received_version }
 *   3. Server responds: { type: "pull", changes }
 *   4. Client applies received changes, sends: { type: "ack" }
 *   5. Server sends: { type: "done" } — connection closes.
 *
 * @param {string} syncId       The sync slot identifier.
 * @param {string} syncSecret   Auth secret (sent as first message for auth).
 * @param {number} timeoutMs    Max time to wait for the sync cycle (default 15s).
 * @returns {Promise<string>}   Outcome: "synced" | "no_changes" | "offline" | "error:<msg>"
 */
export async function runSyncCycle(syncId, syncSecret, timeoutMs = 15000) {
  try {
    getDb(); // Validate DB is registered before proceeding.
    const siteId = await getSiteId();

    // Determine what to send.
    const lastSent = getLastSentVersion();
    const localChanges = await getChangesSince(lastSent);
    const lastReceived = getLastReceivedVersion();

    const wsUrl = buildWsUrl(syncId);
    console.log(`[Sync] Opening WebSocket to ${wsUrl}`);

    return await new Promise((resolve) => {
      let settled = false;
      const timer = setTimeout(() => {
        if (!settled) {
          settled = true;
          try { ws.close(); } catch { /* ignore */ }
          resolve("offline");
        }
      }, timeoutMs);

      let ws;
      try {
        ws = new WebSocket(wsUrl);
      } catch (e) {
        clearTimeout(timer);
        console.warn("[Sync] WebSocket constructor failed:", e);
        resolve("offline");
        return;
      }

      ws.onopen = () => {
        console.log("[Sync] WebSocket connected");
        // Send auth + changes in one message.
        const payload = {
          type: "push",
          sync_secret: syncSecret,
          site_id: Array.from(siteId instanceof Uint8Array ? siteId : []),
          changes: JSON.parse(encodeChanges(localChanges)),
          last_received_version: lastReceived.toString(),
        };
        ws.send(JSON.stringify(payload));
      };

      ws.onmessage = async (event) => {
        try {
          const msg = JSON.parse(event.data);

          if (msg.type === "pull") {
            // Apply remote changes.
            const remoteChanges = msg.changes || [];
            if (remoteChanges.length > 0) {
              console.log(`[Sync] Applying ${remoteChanges.length} remote changes`);
              await applyChanges(remoteChanges);

              // Update last-received version from the server's report.
              if (msg.server_db_version) {
                setLastReceivedVersion(BigInt(msg.server_db_version));
              }
            }

            // Update last-sent version now that the server has our changes.
            const currentVersion = await getDbVersion();
            setLastSentVersion(currentVersion);

            // Acknowledge.
            ws.send(JSON.stringify({ type: "ack" }));
          } else if (msg.type === "done") {
            clearTimeout(timer);
            if (!settled) {
              settled = true;
              ws.close();
              const hadChanges =
                localChanges.length > 0 ||
                (msg.applied_count && msg.applied_count > 0);
              resolve(hadChanges ? "synced" : "no_changes");
            }
          } else if (msg.type === "error") {
            clearTimeout(timer);
            if (!settled) {
              settled = true;
              ws.close();
              resolve(`error:${msg.message || "unknown"}`);
            }
          }
        } catch (e) {
          console.error("[Sync] Error processing message:", e);
          clearTimeout(timer);
          if (!settled) {
            settled = true;
            try { ws.close(); } catch { /* ignore */ }
            resolve(`error:${e.message || "parse error"}`);
          }
        }
      };

      ws.onerror = (event) => {
        console.warn("[Sync] WebSocket error:", event);
        clearTimeout(timer);
        if (!settled) {
          settled = true;
          resolve("offline");
        }
      };

      ws.onclose = (event) => {
        clearTimeout(timer);
        if (!settled) {
          settled = true;
          if (event.code === 1000 || event.code === 1005) {
            resolve("no_changes");
          } else {
            resolve("offline");
          }
        }
      };
    });
  } catch (e) {
    console.error("[Sync] runSyncCycle failed:", e);
    return `error:${e.message || "unknown"}`;
  }
}

/**
 * Check whether the sync server is reachable (lightweight HTTP health check).
 * Falls back to "offline" on any error.
 *
 * @param {string} syncId  The sync slot identifier.
 * @returns {Promise<boolean>} true if the server responded.
 */
export async function checkSyncServerHealth(syncId) {
  try {
    let base = window.SYNC_BASE_URL || "";
    if (!base || base.includes("%%")) {
      base = window.location.origin + "/api";
    }
    const url = `${base.replace(/\/$/, "")}/sync/${syncId}/metadata`;
    const resp = await fetch(url, {
      method: "GET",
      mode: "cors",
      signal: AbortSignal.timeout(5000),
    });
    // 404 is fine — means the slot doesn't exist yet but the server is up.
    return resp.ok || resp.status === 404;
  } catch {
    return false;
  }
}
