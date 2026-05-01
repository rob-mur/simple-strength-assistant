// Module-level singletons to avoid constructing in hot paths.
const _textEncoder = new TextEncoder();
const _textDecoder = new TextDecoder();

// Schema identity — must match sync-backend/schemas/default via cryb64 hash.
const SCHEMA_NAME = "default";
const SCHEMA_VERSION = 7730284589046626158n;

// WebSocket-based CRR changeset sync module using the vlcn.io binary wire protocol.
//
// Uses the crsql_changes() virtual table provided by crsqlite-wasm to extract
// and apply changesets.  Communicates with the sync server (powered by
// @vlcn.io/ws-server) over a WebSocket connection (one per sync cycle).
//
// The server expects:
//   - URL path: /sync/<sync_id> (no /ws suffix)
//   - Room metadata in sec-websocket-protocol header (base64-encoded)
//   - Binary wire format using lib0-compatible encoding (@vlcn.io/ws-common)
//
// Protocol flow:
//   1. Client sends AnnouncePresence (tag=1) with site_id and lastSeens
//   2. Server sends Changes (tag=2) or StartStreaming (tag=4)
//   3. Client sends Changes (tag=2) with local changesets
//
// This module is called from Rust via wasm_bindgen FFI (see src/sync/ws_bridge.rs).
// Sync results are communicated back to Rust through resolved promises.

// ── lib0-compatible binary encoder/decoder ────────────────────────────────────
// Minimal implementation of the lib0 encoding used by @vlcn.io/ws-common.

function createEncoder() {
  return { bufs: [], len: 0 };
}

function _push(enc, bytes) {
  enc.bufs.push(bytes);
  enc.len += bytes.length;
}

function writeUint8(enc, val) {
  _push(enc, new Uint8Array([val & 0xff]));
}

function writeVarUint(enc, val) {
  while (val > 0x7f) {
    _push(enc, new Uint8Array([(val & 0x7f) | 0x80]));
    val = Math.floor(val / 128);
  }
  _push(enc, new Uint8Array([val & 0x7f]));
}

function writeVarInt(enc, val) {
  const isNeg = val < 0;
  if (isNeg) val = -val;
  _push(enc, new Uint8Array([((val > 0x3f ? 0x80 : 0) | (isNeg ? 0x40 : 0) | (val & 0x3f))]));
  val = Math.floor(val / 64);
  while (val > 0) {
    _push(enc, new Uint8Array([(val > 0x7f ? 0x80 : 0) | (val & 0x7f)]));
    val = Math.floor(val / 128);
  }
}

function writeBigInt64(enc, val) {
  const buf = new ArrayBuffer(8);
  // lib0 uses big-endian for BigInt64. Handle unsigned values > 2^63 by
  // wrapping to the signed representation (same bit pattern).
  let signed = BigInt(val);
  if (signed >= (1n << 63n)) signed -= (1n << 64n);
  new DataView(buf).setBigInt64(0, signed, false); // big-endian, matching lib0
  _push(enc, new Uint8Array(buf));
}

function writeFloat64(enc, val) {
  const buf = new ArrayBuffer(8);
  new DataView(buf).setFloat64(0, val, false);
  _push(enc, new Uint8Array(buf));
}

function writeVarString(enc, str) {
  const bytes = _textEncoder.encode(str);
  writeVarUint(enc, bytes.length);
  _push(enc, bytes);
}

function writeUint8Array(enc, arr) {
  _push(enc, new Uint8Array(arr));
}

function writeVarUint8Array(enc, arr) {
  writeVarUint(enc, arr.length);
  _push(enc, new Uint8Array(arr));
}

function toUint8Array(enc) {
  const result = new Uint8Array(enc.len);
  let offset = 0;
  for (const buf of enc.bufs) {
    result.set(buf, offset);
    offset += buf.length;
  }
  return result;
}

// ── Decoder ──

function createDecoder(data) {
  return { data: new Uint8Array(data), pos: 0 };
}

function readUint8(dec) {
  if (dec.pos >= dec.data.length) throw new Error('[Sync] Decoder overrun');
  return dec.data[dec.pos++];
}

function readVarUint(dec) {
  // Use multiplication instead of bitwise OR to avoid 32-bit overflow.
  let val = 0;
  let mult = 1;
  while (true) {
    const b = dec.data[dec.pos++];
    val += (b & 0x7f) * mult;
    if (b < 0x80) break;
    mult *= 128;
  }
  return val;
}

function readVarInt(dec) {
  const first = dec.data[dec.pos++];
  const isNeg = (first & 0x40) !== 0;
  let val = first & 0x3f;
  let mult = 64;
  if (first & 0x80) {
    while (true) {
      const b = dec.data[dec.pos++];
      val += (b & 0x7f) * mult;
      mult *= 128;
      if (b < 0x80) break;
    }
  }
  return isNeg ? -val : val;
}

function readBigInt64(dec) {
  const view = new DataView(dec.data.buffer, dec.data.byteOffset + dec.pos, 8);
  dec.pos += 8;
  return view.getBigInt64(0, false); // big-endian, matching lib0
}

function readFloat64(dec) {
  const view = new DataView(dec.data.buffer, dec.data.byteOffset + dec.pos, 8);
  dec.pos += 8;
  return view.getFloat64(0, false);
}

function readVarString(dec) {
  const len = readVarUint(dec);
  const bytes = dec.data.slice(dec.pos, dec.pos + len);
  dec.pos += len;
  return _textDecoder.decode(bytes);
}

function readUint8ArrayFixed(dec, len) {
  const arr = dec.data.slice(dec.pos, dec.pos + len);
  dec.pos += len;
  return arr;
}

function readVarUint8Array(dec) {
  const len = readVarUint(dec);
  const arr = dec.data.slice(dec.pos, dec.pos + len);
  dec.pos += len;
  return arr;
}

// hasMoreData removed — not needed for current message decoding.

// ── Message tags (from @vlcn.io/ws-common) ────────────────────────────────────

const Tag = {
  AnnouncePresence: 1,
  Changes: 2,
  RejectChanges: 3,
  StartStreaming: 4,
  Ping: 7,
  Pong: 8,
};

// ── Value type tags for encoding change values ────────────────────────────────

const ValType = {
  NULL: 0,
  BIGINT: 1,
  NUMBER: 2,
  STRING: 3,
  BOOL: 4,
  BLOB: 5,
};

// ── Protocol message encoding ─────────────────────────────────────────────────

/**
 * Encode an AnnouncePresence message (tag=1).
 *
 * @param {Uint8Array} siteId   16-byte site identifier from crsql_site_id().
 * @param {Array}      lastSeens Array of [siteId(Uint8Array), [version(bigint), seq(number)]].
 * @returns {Uint8Array} Binary-encoded message.
 */
function encodeAnnouncePresence(siteId, lastSeens) {
  const enc = createEncoder();
  writeUint8(enc, Tag.AnnouncePresence);
  // sender (16 bytes, fixed)
  writeUint8Array(enc, siteId);
  // lastSeens count
  writeVarUint(enc, lastSeens.length);
  for (const [sid, [version, seq]] of lastSeens) {
    writeUint8Array(enc, sid); // 16 bytes
    writeBigInt64(enc, version);
    writeVarInt(enc, seq);
  }
  // schemaName
  writeVarString(enc, SCHEMA_NAME);
  // schemaVersion
  writeBigInt64(enc, SCHEMA_VERSION);
  return toUint8Array(enc);
}

/**
 * Write a typed value to the encoder, prefixed with its type tag.
 */
function writeTypedValue(enc, val) {
  if (val === null || val === undefined) {
    writeUint8(enc, ValType.NULL);
  } else if (typeof val === "bigint") {
    writeUint8(enc, ValType.BIGINT);
    writeBigInt64(enc, val);
  } else if (typeof val === "number") {
    writeUint8(enc, ValType.NUMBER);
    writeFloat64(enc, val);
  } else if (typeof val === "string") {
    writeUint8(enc, ValType.STRING);
    writeVarString(enc, val);
  } else if (typeof val === "boolean") {
    writeUint8(enc, ValType.BOOL);
    writeUint8(enc, val ? 1 : 0);
  } else if (val instanceof Uint8Array) {
    writeUint8(enc, ValType.BLOB);
    writeVarUint8Array(enc, val);
  } else {
    // Fallback: coerce to string
    writeUint8(enc, ValType.STRING);
    writeVarString(enc, String(val));
  }
}

/**
 * Read a typed value from the decoder.
 */
function readTypedValue(dec) {
  const type = readUint8(dec);
  switch (type) {
    case ValType.NULL: return null;
    case ValType.BIGINT: return readBigInt64(dec);
    case ValType.NUMBER: return readFloat64(dec);
    case ValType.STRING: return readVarString(dec);
    case ValType.BOOL: return readUint8(dec) !== 0;
    case ValType.BLOB: return readVarUint8Array(dec);
    default: throw new Error(`[Sync] Unknown value type tag: ${type}`);
  }
}

/**
 * Encode a Changes message (tag=2).
 *
 * @param {Uint8Array} siteId  16-byte site identifier.
 * @param {bigint}     since   The db_version we are sending changes since.
 * @param {Array}      changes Array of change rows from crsql_changes().
 *   Each row: [table, pk, cid, val, col_version, db_version, site_id, cl, seq]
 * @returns {Uint8Array} Binary-encoded message.
 */
function encodeChangesMsg(siteId, since, changes) {
  const enc = createEncoder();
  writeUint8(enc, Tag.Changes);
  // sender
  writeUint8Array(enc, siteId);
  // since: [version, seq] pair — matches vlcn.io wire format
  writeBigInt64(enc, since);
  writeVarInt(enc, 0); // seq component of since (0 = send all from this version)
  // number of changes
  writeVarUint(enc, changes.length);
  for (const row of changes) {
    // [table, pk, cid, val, col_version, db_version, site_id, cl, seq]
    const [table, pk, cid, val, colVersion, dbVersion, changeSiteId, cl, seq] = row;
    writeVarString(enc, table);
    // pk is a packed blob from crsqlite
    writeVarUint8Array(enc, pk instanceof Uint8Array ? pk : _textEncoder.encode(String(pk)));
    writeVarString(enc, cid);
    writeTypedValue(enc, val);
    writeBigInt64(enc, BigInt(colVersion));
    writeBigInt64(enc, BigInt(dbVersion));
    // site_id: NULL tag (0) for null, BLOB tag (5) + raw 16 bytes otherwise
    if (changeSiteId != null && changeSiteId.length > 0) {
      writeUint8(enc, 5); // BLOB
      const sid = changeSiteId instanceof Uint8Array ? changeSiteId : new Uint8Array(changeSiteId);
      if (sid.length !== 16) throw new Error(`[Sync] change site_id has wrong length: ${sid.length}`);
      writeUint8Array(enc, sid); // raw 16 bytes, no length prefix
    } else {
      writeUint8(enc, 0); // NULL
    }
    writeBigInt64(enc, BigInt(cl));
    writeVarInt(enc, Number(seq));
  }
  return toUint8Array(enc);
}

/**
 * Decode an incoming binary message from the server.
 *
 * @param {ArrayBuffer} data  Raw binary message.
 * @returns {{ tag: number, changes?: Array, sender?: Uint8Array }}
 */
function decodeMessage(data) {
  const dec = createDecoder(data);
  const tag = readUint8(dec);

  switch (tag) {
    case Tag.Changes: {
      const sender = readUint8ArrayFixed(dec, 16);
      const sinceVersion = readBigInt64(dec);
      const sinceSeq = readVarInt(dec);
      const count = readVarUint(dec);
      const changes = [];
      for (let i = 0; i < count; i++) {
        const table = readVarString(dec);
        const pk = readVarUint8Array(dec);
        const cid = readVarString(dec);
        const val = readTypedValue(dec);
        const colVersion = readBigInt64(dec);
        const dbVersion = readBigInt64(dec);
        // site_id: NULL tag (0) means null, BLOB tag (5) means 16 raw bytes
        const siteIdTag = readUint8(dec);
        const siteId = siteIdTag === 0 ? null : readUint8ArrayFixed(dec, 16);
        const cl = readBigInt64(dec);
        const seq = readVarInt(dec);
        changes.push([table, pk, cid, val, colVersion, dbVersion, siteId, cl, seq]);
      }
      return { tag, sender, since: [sinceVersion, sinceSeq], changes };
    }

    case Tag.StartStreaming:
      return { tag };

    case Tag.RejectChanges:
      return { tag };

    case Tag.Pong:
      return { tag };

    case Tag.Ping:
      return { tag };

    default:
      console.warn(`[Sync] Unknown message tag: ${tag}`);
      return { tag };
  }
}

// ── Database handle ───────────────────────────────────────────────────────────

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
 * Get the 16-byte site identifier for this client.
 * Required for the AnnouncePresence message.
 *
 * @returns {Promise<Uint8Array>} The 16-byte site_id.
 */
async function getSiteId() {
  const db = getDb();
  const rows = await db.execA("SELECT crsql_site_id()");
  if (rows && rows.length > 0) {
    const raw = rows[0][0];
    if (raw instanceof Uint8Array) return raw;
    // If returned as hex string, convert
    if (typeof raw === "string") {
      const hex = raw.replace(/-/g, "");
      if (hex.length !== 32) {
        throw new Error(`[Sync] site_id hex has wrong length: ${hex.length} (expected 32)`);
      }
      const bytes = new Uint8Array(16);
      for (let i = 0; i < 16; i++) {
        bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
      }
      return bytes;
    }
  }
  throw new Error("[Sync] Could not determine site_id — database may not be initialized");
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
  // Force crsqlite to flush internal state before querying changes.
  // The crsql_changes virtual table reads from crsqlite's internal tracking
  // tables, which may not reflect recent writes until the current auto-commit
  // transaction completes. Calling crsql_db_version() in a fresh statement
  // ensures any pending writes are visible.
  const dbVer = await db.execA("SELECT crsql_db_version()");
  console.log(`[Sync] Current db_version: ${dbVer?.[0]?.[0]}`);

  const rows = await db.execA(
    "SELECT [table], [pk], [cid], [val], [col_version], [db_version], [site_id], [cl], [seq] " +
    "FROM crsql_changes WHERE db_version > ?",
    [Number(sinceVersion)]
  );
  // Log changeset table breakdown for debugging
  if (rows && rows.length > 0) {
    const tables = {};
    for (const r of rows) { tables[r[0]] = (tables[r[0]] || 0) + 1; }
    console.log(`[Sync] getChangesSince(${sinceVersion}): ${rows.length} changes — ${JSON.stringify(tables)}`);
  } else {
    console.log(`[Sync] getChangesSince(${sinceVersion}): 0 changes`);
  }
  return rows || [];
}

/**
 * Apply remote changesets received from the server.
 *
 * Each change is inserted into crsql_changes() which handles CRDT merge logic.
 *
 * @param {Array<Array>} changes  Array of change rows (same column order as getChangesSince).
 */
async function applyChanges(changes, sender) {
  if (!changes || changes.length === 0) return;

  const db = getDb();
  // NOTE: crsqlite requires INSERT INTO crsql_changes to run outside an
  // explicit transaction (auto-commit mode). Wrapping in BEGIN/COMMIT
  // causes the virtual table to silently discard changes.
  try {
    for (const change of changes) {
      // [table, pk, cid, val, col_version, db_version, site_id, cl, seq]
      // col_version(4), db_version(5), cl(7) are BigInt from the decoder.
      // crsqlite-wasm's exec may not handle BigInt parameters — coerce to
      // Number (safe for realistic version values well within Number.MAX_SAFE_INTEGER).
      //
      // site_id: the server sends NULL as a bandwidth optimization (hub-and-spoke).
      // crsqlite needs a non-NULL site_id to distinguish remote from local
      // changes. Use the message sender's site_id when the per-row value is NULL.
      const siteId = change[6] != null ? change[6] : sender;
      const coerced = [
        change[0],                                       // table (string)
        change[1],                                       // pk (Uint8Array)
        change[2],                                       // cid (string)
        typeof change[3] === "bigint" ? Number(change[3]) : change[3], // val (mixed — coerce BigInt)
        Number(change[4]),                               // col_version
        Number(change[5]),                               // db_version
        siteId,                                          // site_id (Uint8Array)
        Number(change[7]),                               // cl
        change[8],                                       // seq (already Number)
      ];
      await db.exec(
        "INSERT INTO crsql_changes ([table], [pk], [cid], [val], [col_version], [db_version], [site_id], [cl], [seq]) " +
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        coerced
      );
    }
  } catch (e) {
    console.error("[Sync] applyChanges INSERT error:", e);
    throw e;
  }
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
  return `${wsBase.replace(/\/$/, "")}/sync/${syncId}`;
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

/**
 * Run one sync cycle over WebSocket using the vlcn.io binary wire protocol.
 *
 * Protocol flow:
 *   1. On open: send AnnouncePresence (tag=1) with our site_id and empty lastSeens
 *   2. Receive: server sends Changes (tag=2) with remote changesets, or
 *      StartStreaming (tag=4) indicating it has no changes
 *   3. Send: Changes (tag=2) with our local changesets since lastSent
 *   4. Close: update lastSentVersion only on successful close (code 1000)
 *
 * @param {string} syncId       The sync slot identifier.
 * @param {string} _syncSecret  Auth secret (currently unused by server).
 * @param {number} timeoutMs    Max time to wait for the sync cycle (default 15s).
 * @returns {Promise<string>}   Outcome: "synced" | "no_changes" | "offline" | "error:<msg>"
 */
export async function runSyncCycle(syncId, _syncSecret, timeoutMs = 15000) {
  try {
    getDb(); // Validate DB is registered before proceeding.

    // Gather prerequisites before opening the WebSocket.
    const siteId = await getSiteId();
    const lastSent = getLastSentVersion();
    const localChanges = await getChangesSince(lastSent);

    const wsUrl = buildWsUrl(syncId);
    console.log(`[Sync] Opening WebSocket to ${wsUrl}`);

    // Encode room info in sec-websocket-protocol as vlcn.io expects.
    // Strip base64 padding (`=`) — it is not allowed in WebSocket subprotocol
    // values per RFC 6455 §4.1.  The vlcn.io server decodes unpadded base64.
    const roomMeta = btoa(`room=${syncId},schemaName=${SCHEMA_NAME},schemaVersion=${SCHEMA_VERSION}`).replace(/=+$/, "");

    return await new Promise((resolve) => {
      let settled = false;
      let receivedRemoteChanges = false;
      let sentLocalChanges = false;
      let versionAtSend = 0n;

      // Deferred close timer — gives in-flight server messages time to arrive
      // and be fully processed before we close the WebSocket.
      let closeTimer = null;

      /**
       * Schedule a graceful close after a short delay.  Any new message
       * arrival resets the timer so that in-progress applyChanges() calls
       * finish before the connection is torn down.
       */
      const scheduleClose = () => {
        if (closeTimer) clearTimeout(closeTimer);
        closeTimer = setTimeout(async () => {
          closeTimer = null;
          // versionAtSend is captured in sendLocalChanges() BEFORE remote
          // changes are applied — don't re-read getDbVersion() here, as it
          // would include the higher version from applied remote changes.
          if (ws.readyState === WebSocket.OPEN) {
            ws.close(1000, "sync complete");
          }
        }, 300);
      };

      const timer = setTimeout(() => {
        if (!settled) {
          settled = true;
          if (closeTimer) clearTimeout(closeTimer);
          try { ws.close(); } catch { /* ignore */ }
          resolve("offline");
        }
      }, timeoutMs);

      let ws;
      try {
        ws = new WebSocket(wsUrl, [roomMeta]);
      } catch (e) {
        clearTimeout(timer);
        console.warn("[Sync] WebSocket constructor failed:", e);
        resolve("offline");
        return;
      }

      // vlcn.io uses binary frames
      ws.binaryType = "arraybuffer";

      ws.onopen = () => {
        console.log("[Sync] WebSocket connected, sending AnnouncePresence");
        try {
          // Send AnnouncePresence with empty lastSeens.
          // TODO(perf): persist lastSeens to localStorage to enable incremental sync.
          const msg = encodeAnnouncePresence(siteId, []);
          ws.send(msg);
        } catch (e) {
          console.error("[Sync] Error sending AnnouncePresence:", e);
          clearTimeout(timer);
          if (closeTimer) clearTimeout(closeTimer);
          if (!settled) {
            settled = true;
            try { ws.close(4000, "announce error"); } catch { /* ignore */ }
            resolve("error:announce failed");
          }
        }
      };

      /**
       * Send our local changes after receiving the server's changes or StartStreaming.
       * Does NOT close the WebSocket — scheduleClose() handles that after a
       * grace period so any remaining server Changes can arrive and be applied.
       */
      const sendLocalChanges = async () => {
        if (sentLocalChanges) return;
        sentLocalChanges = true;
        try {
          if (localChanges.length > 0) {
            console.log(`[Sync] Sending ${localChanges.length} changes with since=${lastSent}`);
            const msg = encodeChangesMsg(siteId, lastSent, localChanges);
            ws.send(msg);
            console.log(`[Sync] Sent ${localChanges.length} local changes (${msg.byteLength} bytes)`);
          }
          // Capture the db_version at send time. When triggered by StartStreaming
          // this is before remote changes; when triggered by the Changes handler,
          // remote changes have already been applied. Either way, this version
          // represents the state the server should track for our next sync.
          try {
            versionAtSend = await getDbVersion();
          } catch { /* ignore */ }
          // Don't close immediately — the server may still be sending Changes
          // from its OutboundStream.  scheduleClose() waits 300ms for any
          // remaining messages before closing.
          scheduleClose();
        } catch (e) {
          console.error("[Sync] Error sending changes:", e);
          if (closeTimer) clearTimeout(closeTimer);
          ws.close(4000, "send error"); // Non-1000 so onclose treats as failure
        }
      };

      ws.onmessage = async (event) => {
        try {
          if (!(event.data instanceof ArrayBuffer)) {
            console.warn("[Sync] Unexpected non-binary message, ignoring");
            return;
          }

          const decoded = decodeMessage(event.data);

          switch (decoded.tag) {
            case Tag.Changes: {
              // Pause the close timer while we apply changes — prevents
              // onclose from firing mid-applyChanges and settling the
              // promise before receivedRemoteChanges is set.
              if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
              if (decoded.changes && decoded.changes.length > 0) {
                const remoteTables = {};
                for (const r of decoded.changes) { remoteTables[r[0]] = (remoteTables[r[0]] || 0) + 1; }
                console.log(`[Sync] Received ${decoded.changes.length} remote changes — ${JSON.stringify(remoteTables)}`);
                await applyChanges(decoded.changes, decoded.sender);
                receivedRemoteChanges = true;
              } else {
                console.log("[Sync] Received empty Changes message");
              }
              // After receiving changes, send ours (or reschedule close)
              if (!sentLocalChanges) {
                await sendLocalChanges();
              } else {
                scheduleClose();
              }
              break;
            }

            case Tag.StartStreaming: {
              console.log("[Sync] Server ready for streaming, sending local changes");
              await sendLocalChanges();
              break;
            }

            case Tag.RejectChanges: {
              console.warn("[Sync] Server rejected our changes");
              clearTimeout(timer);
              if (closeTimer) clearTimeout(closeTimer);
              if (!settled) {
                settled = true;
                ws.close(4001, "changes rejected");
                resolve("error:changes rejected");
              }
              break;
            }

            case Tag.Pong:
            case Tag.Ping: {
              // Respond to ping with pong
              if (decoded.tag === Tag.Ping) {
                const pong = createEncoder();
                writeUint8(pong, Tag.Pong);
                ws.send(toUint8Array(pong));
              }
              break;
            }

            default:
              console.warn(`[Sync] Unhandled message tag: ${decoded.tag}`);
          }
        } catch (e) {
          console.error("[Sync] Error processing message:", e);
          clearTimeout(timer);
          if (closeTimer) clearTimeout(closeTimer);
          if (!settled) {
            settled = true;
            try { ws.close(); } catch { /* ignore */ }
            resolve(`error:${e.message || "decode error"}`);
          }
        }
      };

      ws.onerror = (event) => {
        console.warn("[Sync] WebSocket error:", event);
        clearTimeout(timer);
        if (closeTimer) clearTimeout(closeTimer);
        if (!settled) {
          settled = true;
          resolve("offline");
        }
      };

      ws.onclose = (event) => {
        clearTimeout(timer);
        if (closeTimer) clearTimeout(closeTimer);
        if (!settled) {
          settled = true;
          if (event.code === 1000) {
            // Only update lastSentVersion on successful close.
            // versionAtSend was captured in sendLocalChanges() BEFORE remote
            // changes were applied, so it reflects only our local state.
            if (sentLocalChanges && versionAtSend > 0n) {
              console.log(`[Sync] Setting lastSentVersion to ${versionAtSend}`);
              setLastSentVersion(versionAtSend);
            }
            const hadChanges = localChanges.length > 0 || receivedRemoteChanges;
            resolve(hadChanges ? "synced" : "no_changes");
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
 * @param {string} syncId  The sync slot identifier (unused — health is global).
 * @returns {Promise<boolean>} true if the server responded.
 */
export async function checkSyncServerHealth(_syncId) {
  try {
    let base = window.SYNC_BASE_URL || "";
    if (!base || base.includes("%%")) {
      base = window.location.origin + "/api";
    }
    const url = `${base.replace(/\/$/, "")}/health`;
    const resp = await fetch(url, {
      method: "GET",
      mode: "cors",
      signal: AbortSignal.timeout(5000),
    });
    return resp.ok;
  } catch {
    return false;
  }
}
