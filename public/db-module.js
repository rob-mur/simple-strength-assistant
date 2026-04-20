// @vlcn.io/crsqlite-wasm database module.
//
// Replaces the previous sql.js-based implementation. Uses crsqlite-wasm for
// CRR (Conflict-free Replicated Relations) support. On first launch after the
// update, existing OPFS data is migrated in-place by reading it with a
// temporary sql.js instance and inserting the rows into the new crsqlite DB.
// After migration the database is persisted via IndexedDB (IDBBatchAtomicVFS).

// Vendored @vlcn.io/crsqlite-wasm@0.16.0 — no external runtime dependency.
// The .mjs bundle and .wasm binary live in public/vendor/crsqlite/.
const CRSQLITE_WASM_URL = "/vendor/crsqlite/crsqlite-wasm.mjs";
const DB_NAME = "workout-data";

// Tables that must be marked as CRRs for CRDT-based replication.
const CRR_TABLES = ["exercises", "completed_sets", "settings"];

// Migration sentinel key — checked in both localStorage (legacy) and in the DB
// itself (new: stored atomically with the migrated data).
const MIGRATION_KEY = "crsqlite_migration_done";

let db = null;
let sqlite = null;

// Lazy-import of the sync module. Resolved on first use.
let _syncModulePromise = null;
async function getSyncModule() {
  if (!_syncModulePromise) {
    _syncModulePromise = import("./sync-module.js");
  }
  return _syncModulePromise;
}

/**
 * Register the open database handle with the sync module so it can read/write
 * changesets via crsql_changes().
 */
async function registerDbWithSync() {
  try {
    const syncMod = await getSyncModule();
    syncMod.registerSyncDb(db);
    console.log("[DB] Registered database with sync module");
  } catch (e) {
    console.warn("[DB] Failed to register database with sync module:", e);
  }
}

/**
 * Initialise the crsqlite-wasm module (singleton).
 */
async function ensureCrSQLiteLoaded() {
  if (sqlite) return sqlite;

  const mod = await import(CRSQLITE_WASM_URL);
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
    // The sentinel is stored inside the DB itself (in a _meta table) so it is
    // committed atomically with the migrated data.  We also check localStorage
    // for backwards compat with any user who already migrated before this fix.
    const alreadyMigrated =
      localStorage.getItem(MIGRATION_KEY) ||
      (await isMigrationDone());

    const needsMigration =
      fileData && fileData.length > 0 && !alreadyMigrated;

    if (needsMigration) {
      console.log("[DB] Migrating existing OPFS data into crsqlite...");
      await migrateFromSqlJs(new Uint8Array(fileData));
      // Also set localStorage so the check short-circuits on next launch.
      localStorage.setItem(MIGRATION_KEY, Date.now().toString());
      console.log("[DB] Migration complete.");
    }

    // ── Idempotent CRR upgrade ─────────────────────────────────────────────
    // Safe to run every launch — crsql_as_crr is a no-op on tables that are
    // already CRRs.
    await applyCrrMigration();

    // Register the open DB handle with the sync module so WebSocket sync
    // can access crsql_changes().
    await registerDbWithSync();

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
 *
 * Note: `WITH` (CTE) queries are always routed through the mutation path to
 * avoid misrouting CTE-INSERT/UPDATE/DELETE as SELECTs.  If you need a
 * CTE-SELECT, rewrite it without WITH or add RETURNING.
 */
export async function executeQuery(sql, params) {
  if (!db) {
    throw new Error("Database not initialized");
  }

  try {
    const trimmed = sql.trimStart().toUpperCase();

    // Determine whether this statement returns rows.
    const hasReturning = /\bRETURNING\b/i.test(sql);

    // A WITH (CTE) clause returns rows only when the final statement is a
    // SELECT.  Parsing CTEs reliably (nested parens, subqueries) is hard, so
    // we use a conservative approach: treat WITH as a mutation unless
    // RETURNING is present.  All CTE-SELECT queries in this app can be
    // rewritten without WITH if needed; misrouting a mutation through execO
    // would cause silent data loss.
    const returnsRows =
      hasReturning ||
      trimmed.startsWith("SELECT") ||
      trimmed.startsWith("PRAGMA") ||
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

/**
 * Export the crsqlite-wasm database as a standard SQLite file (Uint8Array).
 *
 * crsqlite-wasm stores data in IndexedDB and doesn't expose a raw `.export()`
 * method like sql.js did. To produce a portable `.sqlite` file we:
 *   1. Read every user table from the live crsqlite DB via SELECT queries.
 *   2. Create a temporary sql.js in-memory database.
 *   3. Replicate schema + rows into the sql.js instance.
 *   4. Call sql.js `.export()` to obtain the raw bytes.
 *
 * sql.js is already shipped in `public/` for the OPFS migration path.
 *
 * @returns {Promise<Uint8Array>} The raw bytes of a standard SQLite database.
 */
export async function exportDatabase() {
  if (!db) {
    throw new Error("Database not initialized");
  }

  // Ensure sql.js is loaded (it's still shipped in public/ for migration).
  if (typeof window.initSqlJs === "undefined") {
    await loadScript("sql-wasm.js");
  }
  const SQL = await window.initSqlJs({ locateFile: (file) => file });
  const outDb = new SQL.Database();

  // Discover user tables (skip sqlite internals, crsqlite internals, _meta).
  const tables = await db.execO(
    "SELECT name, sql FROM sqlite_master WHERE type='table' " +
      "AND name NOT LIKE 'sqlite_%' " +
      "AND name NOT LIKE '__crsql_%' " +
      "AND name NOT LIKE 'crsql_%' " +
      "AND name != '_meta'",
  );

  for (const { name, sql: createSql } of tables) {
    if (!createSql) continue;

    // Create the table in the output database.
    const safeCreate = createSql.replace(
      /CREATE\s+TABLE\s+/i,
      "CREATE TABLE IF NOT EXISTS ",
    );
    outDb.run(safeCreate);

    // Copy rows.
    const rows = await db.execO(`SELECT * FROM "${name}"`);
    if (rows.length === 0) continue;

    const cols = Object.keys(rows[0]);
    const placeholders = cols.map(() => "?").join(", ");
    const insertSql = `INSERT INTO "${name}" (${cols.map((c) => `"${c}"`).join(", ")}) VALUES (${placeholders})`;
    const stmt = outDb.prepare(insertSql);
    for (const row of rows) {
      stmt.run(cols.map((c) => row[c]));
    }
    stmt.free();
  }

  // Also copy indexes (including UNIQUE indexes).
  const indexes = await db.execO(
    "SELECT sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL " +
      "AND name NOT LIKE '__crsql_%' " +
      "AND name NOT LIKE 'crsql_%'",
  );
  for (const { sql: idxSql } of indexes) {
    if (idxSql) {
      try {
        const safeIdx = idxSql.replace(
          /CREATE\s+(UNIQUE\s+)?INDEX\s+/i,
          "CREATE $1INDEX IF NOT EXISTS ",
        );
        outDb.run(safeIdx);
      } catch (e) {
        console.warn("[DB] Failed to copy index:", e);
      }
    }
  }

  // Copy the user_version pragma so migrations detect the correct version.
  const versionRows = await db.execA("PRAGMA user_version");
  if (versionRows && versionRows.length > 0) {
    outDb.run(`PRAGMA user_version = ${versionRows[0][0]}`);
  }

  const bytes = outDb.export();
  outDb.close();
  return bytes;
}

/**
 * Import a user-supplied SQLite file into the crsqlite database.
 *
 * Unlike `initDatabase`, this always loads the provided bytes regardless of
 * whether the one-time OPFS migration has already run.  It closes the current
 * database, opens a fresh one, and migrates the supplied bytes via sql.js.
 *
 * @param {Array|Uint8Array} fileData  Raw bytes of a SQLite database to import.
 * @returns {Promise<boolean>} true on success.
 */
export async function importDatabase(fileData) {
  try {
    if (!fileData || fileData.length === 0) {
      console.error("[DB] importDatabase called with empty data");
      return false;
    }

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

    console.log("[DB] Importing user-supplied database...");
    // The import is wrapped in a single transaction (inside
    // migrateFromSqlJs) so that DELETEs + INSERTs are atomic — if the
    // migration fails, the user's existing data is rolled back intact.
    await migrateFromSqlJs(new Uint8Array(fileData), { clearFirst: true });
    console.log("[DB] Import complete.");

    // Re-apply CRR upgrade on the freshly imported data.
    await applyCrrMigration();

    // Re-register with sync module after import.
    await registerDbWithSync();

    return true;
  } catch (error) {
    console.error("Failed to import database:", error);
    return false;
  }
}

// ── Migration sentinel helpers ────────────────────────────────────────────────

/**
 * Check whether the one-time OPFS migration has already been committed.
 * The sentinel lives in a `_meta` key-value table inside the crsqlite DB so
 * that it is written atomically with the migrated data.
 */
async function isMigrationDone() {
  try {
    const rows = await db.execO(
      "SELECT value FROM _meta WHERE key = ?",
      [MIGRATION_KEY],
    );
    return rows.length > 0;
  } catch {
    // Table doesn't exist yet — migration has never run.
    return false;
  }
}

/**
 * Record the migration sentinel inside the current transaction.
 * Must be called while a BEGIN…COMMIT is active.
 */
async function setMigrationDone() {
  await db.exec(
    "CREATE TABLE IF NOT EXISTS _meta (key TEXT PRIMARY KEY, value TEXT)",
  );
  await db.exec("INSERT OR REPLACE INTO _meta (key, value) VALUES (?, ?)", [
    MIGRATION_KEY,
    Date.now().toString(),
  ]);
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
        await db.exec("SELECT crsql_as_crr(?)", [table]);
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
async function migrateFromSqlJs(fileData, { clearFirst = false } = {}) {
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

  // Wrap the entire migration in a transaction for atomicity and performance.
  await db.exec("BEGIN");
  try {
    // When importing user data, clear existing tables inside the transaction
    // so that a failure rolls back both the DELETEs and the INSERTs.
    if (clearFirst) {
      for (const table of CRR_TABLES) {
        try {
          await db.exec(`DELETE FROM "${table}"`);
        } catch {
          // Table may not exist yet — migration will create it.
        }
      }
    }

    for (const table of tables) {
      // Get the CREATE TABLE statement so we can replicate the schema,
      // even for tables with zero rows (e.g. `settings` on a fresh install).
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

      // Read all rows from the old database.
      const rows = [];
      const stmt = oldDb.prepare(`SELECT * FROM "${table}"`);
      while (stmt.step()) {
        rows.push(stmt.getAsObject());
      }
      stmt.free();

      if (rows.length === 0) {
        console.log(`[DB] Replicated schema for empty table '${table}'`);
        continue;
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

    // Also migrate indexes (including UNIQUE indexes).
    const idxStmt = oldDb.prepare(
      "SELECT sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL"
    );
    while (idxStmt.step()) {
      const idxSql = idxStmt.getAsObject().sql;
      if (idxSql) {
        const safeIdx = idxSql.replace(
          /CREATE\s+(UNIQUE\s+)?INDEX\s+/i,
          "CREATE $1INDEX IF NOT EXISTS "
        );
        try {
          await db.exec(safeIdx);
        } catch (e) {
          console.warn("[DB] Failed to migrate index:", e);
        }
      }
    }
    idxStmt.free();

    // Store the migration sentinel inside the same transaction so it commits
    // atomically with the migrated data.
    await setMigrationDone();

    await db.exec("COMMIT");
  } catch (migrationError) {
    await db.exec("ROLLBACK");
    throw migrationError;
  } finally {
    oldDb.close();
  }
}

/**
 * Dynamically load a script by inserting a <script> tag.
 * Returns a promise that resolves when the script has loaded.
 * Deduplicates: concurrent calls for the same src share one promise.
 */
const _loadingScripts = new Map();
function loadScript(src) {
  if (_loadingScripts.has(src)) return _loadingScripts.get(src);
  const promise = new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = src;
    script.onload = () => {
      _loadingScripts.delete(src);
      resolve();
    };
    script.onerror = (err) => {
      _loadingScripts.delete(src);
      reject(err);
    };
    document.head.appendChild(script);
  });
  _loadingScripts.set(src, promise);
  return promise;
}
