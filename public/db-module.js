// @vlcn.io/crsqlite-wasm database module.
//
// Replaces the previous sql.js-based implementation. Uses crsqlite-wasm for
// CRR (Conflict-free Replicated Relations) support. On first launch after the
// update, existing OPFS data is migrated in-place by reading it with a
// temporary sql.js instance and inserting the rows into the new crsqlite DB.
// After migration the database is persisted via IndexedDB (IDBBatchAtomicVFS).

const CRSQLITE_CDN =
  "https://esm.sh/@vlcn.io/crsqlite-wasm@0.16.0";
const DB_NAME = "workout-data";

// Tables that must be marked as CRRs for CRDT-based replication.
const CRR_TABLES = ["exercises", "completed_sets", "settings"];

// Migration sentinel: stored in localStorage after a successful OPFS→crsqlite
// migration so it never runs twice.
const MIGRATION_KEY = "crsqlite_migration_done";

let db = null;
let sqlite = null;

/**
 * Initialise the crsqlite-wasm module (singleton).
 */
async function ensureCrSQLiteLoaded() {
  if (sqlite) return sqlite;

  const mod = await import(CRSQLITE_CDN);
  const initWasm = mod.default;
  sqlite = await initWasm();
  return sqlite;
}

/**
 * Initialise (or re-initialise) the database.
 *
 * @param {Array|Uint8Array|null} fileData  Raw bytes of an existing SQLite
 *   database (read from OPFS by the Rust layer). When non-empty, the data is
 *   migrated into the new crsqlite DB on first launch.
 * @returns {Promise<boolean>} true on success.
 */
export async function initDatabase(fileData) {
  try {
    await ensureCrSQLiteLoaded();

    // Close any previously open database.
    if (db) {
      try {
        await db.close();
      } catch (e) {
        console.warn("Failed to close existing database:", e);
      }
    }

    db = await sqlite.open(DB_NAME);

    // ── One-time migration from OPFS (sql.js) data ─────────────────────────
    // If the caller provided file data AND we haven't migrated yet, read the
    // old database with sql.js and replay its rows into the new crsqlite DB.
    const needsMigration =
      fileData &&
      fileData.length > 0 &&
      !localStorage.getItem(MIGRATION_KEY);

    if (needsMigration) {
      console.log("[DB] Migrating existing OPFS data into crsqlite...");
      await migrateFromSqlJs(new Uint8Array(fileData));
      localStorage.setItem(MIGRATION_KEY, Date.now().toString());
      console.log("[DB] Migration complete.");
    }

    // ── Idempotent CRR upgrade ─────────────────────────────────────────────
    // Safe to run every launch — crsql_as_crr is a no-op on tables that are
    // already CRRs.
    await applyCrrMigration();

    // Expose a raw SQL hook for the Playwright test harness.
    if (typeof window !== "undefined" && window.__TEST_MODE__) {
      window.__dbExecuteQuery = (sql, params) => executeQuery(sql, params);
    }

    return true;
  } catch (error) {
    console.error("Failed to initialize database:", error);
    return false;
  }
}

/**
 * Execute a SQL statement and return its results.
 *
 * For statements that return rows (SELECT, PRAGMA, etc.) the return value is
 * an array of plain objects (one per row, keys = column names).
 *
 * For statements that do not return rows (INSERT, UPDATE, DELETE, CREATE, …)
 * the return value is `{ changes: <number> }`.
 */
export async function executeQuery(sql, params) {
  if (!db) {
    throw new Error("Database not initialized");
  }

  try {
    const trimmed = sql.trimStart().toUpperCase();

    // Determine whether this statement returns rows.
    const hasReturning = /\bRETURNING\b/i.test(sql);
    const returnsRows =
      hasReturning ||
      trimmed.startsWith("SELECT") ||
      trimmed.startsWith("PRAGMA") ||
      trimmed.startsWith("WITH") ||
      trimmed.startsWith("EXPLAIN");

    if (returnsRows) {
      // execO returns an array of objects (column-name keys).
      const rows = await db.execO(sql, params ?? []);
      return rows;
    }

    // Mutation — run it and report the number of changed rows.
    await db.exec(sql, params ?? []);
    const changesResult = await db.execA("SELECT changes()");
    const changes =
      changesResult && changesResult.length > 0 ? changesResult[0][0] : 0;
    return { changes };
  } catch (error) {
    console.error("Query execution failed:", error.message || error);
    throw error;
  }
}

/**
 * Triggers a browser download of the given bytes as a .sqlite file.
 * Works on iOS Safari, Chrome Android, and any browser supporting Blob URLs.
 *
 * @param {Uint8Array} data - The raw bytes of the SQLite database.
 * @param {string} filename - The suggested download filename (e.g. "workout-data.sqlite").
 */
export function triggerSqliteDownload(data, filename) {
  const blob = new Blob([data], { type: "application/x-sqlite3" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Revoke the object URL after a short delay to free memory.
  // 1 s is sufficient for all major browsers to initiate the download.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/**
 * Mark the three sync-relevant tables as CRRs (Conflict-free Replicated
 * Relations). This is idempotent — calling it on a table that is already a
 * CRR is a harmless no-op.
 */
async function applyCrrMigration() {
  for (const table of CRR_TABLES) {
    try {
      // Check if the table exists before trying to mark it as a CRR.
      const info = await db.execO(
        `SELECT name FROM sqlite_master WHERE type='table' AND name=?`,
        [table]
      );
      if (info.length > 0) {
        await db.exec(`SELECT crsql_as_crr('${table}')`);
        console.log(`[DB] Marked '${table}' as CRR`);
      }
    } catch (e) {
      // crsql_as_crr throws if the table is already a CRR in some builds;
      // treat that as success.
      if (
        e.message &&
        (e.message.includes("already") || e.message.includes("CRR"))
      ) {
        console.log(`[DB] '${table}' is already a CRR`);
      } else {
        console.warn(`[DB] Failed to mark '${table}' as CRR:`, e);
      }
    }
  }
}

/**
 * One-time migration: load the old OPFS SQLite bytes via sql.js (dynamically
 * imported), read every row from the known tables, and INSERT them into the
 * new crsqlite database.
 *
 * sql.js is only loaded during this migration and is never used again.
 *
 * @param {Uint8Array} fileData Raw bytes of the old SQLite database.
 */
async function migrateFromSqlJs(fileData) {
  // Dynamically load sql.js — it's still shipped in public/ for this one-time
  // migration but is no longer loaded in index.html.
  if (typeof window.initSqlJs === "undefined") {
    await loadScript("sql-wasm.js");
  }

  const SQL = await window.initSqlJs({ locateFile: (file) => file });
  const oldDb = new SQL.Database(fileData);

  // Discover tables in the old database.
  const tables = [];
  const tableStmt = oldDb.prepare(
    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
  );
  while (tableStmt.step()) {
    tables.push(tableStmt.getAsObject().name);
  }
  tableStmt.free();

  for (const table of tables) {
    // Read all rows from the old database.
    const rows = [];
    const stmt = oldDb.prepare(`SELECT * FROM "${table}"`);
    while (stmt.step()) {
      rows.push(stmt.getAsObject());
    }
    stmt.free();

    if (rows.length === 0) continue;

    // Get the CREATE TABLE statement so we can replicate the schema.
    const schemaStmt = oldDb.prepare(
      "SELECT sql FROM sqlite_master WHERE type='table' AND name=?",
    );
    schemaStmt.bind([table]);
    let createSql = null;
    if (schemaStmt.step()) {
      createSql = schemaStmt.getAsObject().sql;
    }
    schemaStmt.free();

    if (createSql) {
      // Create the table in the new database (IF NOT EXISTS to be safe).
      const safeCreate = createSql.replace(
        /CREATE\s+TABLE\s+/i,
        "CREATE TABLE IF NOT EXISTS "
      );
      await db.exec(safeCreate);
    }

    // Insert rows.
    for (const row of rows) {
      const cols = Object.keys(row);
      const placeholders = cols.map(() => "?").join(", ");
      const values = cols.map((c) => row[c]);
      const insertSql = `INSERT OR IGNORE INTO "${table}" (${cols.map((c) => `"${c}"`).join(", ")}) VALUES (${placeholders})`;
      await db.exec(insertSql, values);
    }

    console.log(`[DB] Migrated ${rows.length} rows from '${table}'`);
  }

  // Also migrate indexes.
  const idxStmt = oldDb.prepare(
    "SELECT sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL"
  );
  while (idxStmt.step()) {
    const idxSql = idxStmt.getAsObject().sql;
    if (idxSql) {
      const safeIdx = idxSql.replace(
        /CREATE\s+INDEX\s+/i,
        "CREATE INDEX IF NOT EXISTS "
      );
      try {
        await db.exec(safeIdx);
      } catch (e) {
        console.warn("[DB] Failed to migrate index:", e);
      }
    }
  }
  idxStmt.free();

  oldDb.close();
}

/**
 * Dynamically load a script by inserting a <script> tag.
 * Returns a promise that resolves when the script has loaded.
 */
function loadScript(src) {
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = src;
    script.onload = resolve;
    script.onerror = reject;
    document.head.appendChild(script);
  });
}
