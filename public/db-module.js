let db = null;
let SQL = null;

async function ensureSQLLoaded() {
  if (SQL) return SQL;

  if (typeof window.initSqlJs === "undefined") {
    throw new Error("sql.js not loaded. Make sure to include it in your HTML.");
  }

  SQL = await window.initSqlJs({
    locateFile: (file) => file,
  });

  return SQL;
}

export async function initDatabase(fileData) {
  try {
    await ensureSQLLoaded();

    // CRITICAL: Close existing database to ensure test isolation
    // Without this, tests share state and see data from previous tests
    if (db) {
      try {
        db.close();
      } catch (e) {
        console.warn("Failed to close existing database:", e);
      }
      db = null;
    }

    if (fileData && fileData.length > 0) {
      const uint8Array = new Uint8Array(fileData);
      db = new SQL.Database(uint8Array);
    } else {
      db = new SQL.Database();
    }

    // Expose a raw SQL hook only when the test harness has flagged this as a
    // test environment (window.__TEST_MODE__ = true is set via addInitScript
    // in the Playwright fixture before the page loads). This ensures the hook
    // is never present in production builds.
    if (typeof window !== "undefined" && window.__TEST_MODE__) {
      window.__dbExecuteQuery = (sql, params) => executeQuery(sql, params);
    }

    return true;
  } catch (error) {
    console.error("Failed to initialize database:", error);
    return false;
  }
}

export async function executeQuery(sql, params) {
  if (!db) {
    throw new Error("Database not initialized");
  }

  let stmt;
  try {
    stmt = db.prepare(sql);
    if (params && params.length > 0) {
      stmt.bind(params);
    }

    const rows = [];
    while (stmt.step()) {
      rows.push(stmt.getAsObject());
    }

    const columnNames = stmt.getColumnNames();
    stmt.free();

    // If the statement returns columns, we return the rows
    // (even if empty, we return []).
    // If it doesn't return columns, we return { changes: db.getRowsModified() }.
    if (columnNames.length > 0) {
      return rows;
    }

    return { changes: db.getRowsModified() };
  } catch (error) {
    if (stmt) {
      try {
        stmt.free();
      } catch (e) {}
    }
    console.error("Query execution failed:", error.message || error);
    throw error;
  }
}

export async function exportDatabase() {
  if (!db) {
    throw new Error("Database not initialized");
  }

  try {
    const uint8Array = db.export();
    return uint8Array;
  } catch (error) {
    console.error("Failed to export database:", error);
    throw error;
  }
}

/**
 * Pure union-merge of two SQLite database blobs.
 *
 * Merge strategy per table, per UUID:
 * - UUID in only one database → insert into the other
 * - UUID in both, different updated_at → higher updated_at wins (all fields)
 * - UUID in both, same updated_at, different field values → true conflict
 *   (the row from A is kept in the merged output as a placeholder)
 * - Soft-deleted records (tombstone: deleted_at set) sync as normal rows;
 *   deleted_at is subject to last-write-wins
 * - Both have a tombstone for the same UUID → merged keeps the more recent deleted_at
 *
 * This function does NOT write to OPFS, does NOT touch the global `db` variable,
 * and produces no side effects.
 *
 * @param {Uint8Array} dataA - Raw bytes of the first SQLite database.
 * @param {Uint8Array} dataB - Raw bytes of the second SQLite database.
 * @returns {{ merged: Uint8Array, conflicts: Array<{uuid, table_name, version_a, version_b}> }}
 */
export async function mergeDatabases(dataA, dataB) {
  const SQL = await ensureSQLLoaded();

  const dbA = new SQL.Database(new Uint8Array(dataA));
  const dbB = new SQL.Database(new Uint8Array(dataB));

  // We build the merged result into a fresh database seeded from A.
  const merged = new SQL.Database(new Uint8Array(dataA));

  const conflicts = [];

  // The tables we know carry UUIDs and sync columns.
  const TABLES = ["exercises", "completed_sets"];

  for (const table of TABLES) {
    // Fetch all rows from B (include tombstones — deleted_at IS NOT filtered).
    let rowsB;
    try {
      rowsB = queryAll(dbB, `SELECT * FROM ${table}`);
    } catch {
      // Table may not exist in B (e.g. brand-new DB with no sets yet).
      continue;
    }

    for (const rowB of rowsB) {
      const uuid = rowB.uuid;
      if (!uuid) continue;

      // Look up the same UUID in the merged (A-seeded) database.
      const existingRows = queryAll(
        merged,
        `SELECT * FROM ${table} WHERE uuid = ?`,
        [uuid]
      );

      if (existingRows.length === 0) {
        // UUID only exists in B — insert it into the merged database.
        insertRow(merged, table, rowB);
        continue;
      }

      const rowA = existingRows[0];

      // Compare updated_at to decide which wins.
      const updatedAtA = rowA.updated_at ?? 0;
      const updatedAtB = rowB.updated_at ?? 0;

      if (updatedAtB > updatedAtA) {
        // B is newer — replace A's row with B's.
        updateRow(merged, table, rowB);
        continue;
      }

      if (updatedAtA > updatedAtB) {
        // A is already the winner — nothing to do.
        continue;
      }

      // Same updated_at — check for tombstone dominance first.
      const deletedAtA = rowA.deleted_at ?? null;
      const deletedAtB = rowB.deleted_at ?? null;

      // If one is a tombstone and the other is live → tombstone wins.
      if (deletedAtA !== null && deletedAtB === null) {
        // A is already tombstoned — nothing to do.
        continue;
      }
      if (deletedAtB !== null && deletedAtA === null) {
        // B is tombstoned, A is live → apply B's tombstone.
        updateRow(merged, table, rowB);
        continue;
      }
      if (deletedAtA !== null && deletedAtB !== null) {
        // Both tombstoned — keep the more recent deleted_at.
        if (deletedAtB > deletedAtA) {
          updateRow(merged, table, rowB);
        }
        continue;
      }

      // Both live, same updated_at — check if field values differ.
      if (!rowsEqual(rowA, rowB)) {
        // True conflict: record it.  Keep A's version in the merged output.
        conflicts.push({
          uuid,
          table_name: table,
          version_a: JSON.stringify(rowA),
          version_b: JSON.stringify(rowB),
        });
      }
      // If rows are identical there is nothing to do.
    }
  }

  const mergedBytes = merged.export();

  dbA.close();
  dbB.close();
  merged.close();

  return { merged: mergedBytes, conflicts };
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/**
 * Execute a SELECT query and return all rows as plain objects.
 * @param {SQL.Database} database
 * @param {string} sql
 * @param {Array} [params]
 * @returns {Array<Object>}
 */
function queryAll(database, sql, params = []) {
  const stmt = database.prepare(sql);
  if (params.length > 0) stmt.bind(params);
  const rows = [];
  while (stmt.step()) {
    rows.push(stmt.getAsObject());
  }
  stmt.free();
  return rows;
}

/**
 * Insert a row (from database B) into the merged database.
 * Columns are taken from the row object; id is excluded so SQLite
 * assigns a new auto-increment value.
 */
function insertRow(database, table, row) {
  const cols = Object.keys(row).filter((c) => c !== "id");
  const placeholders = cols.map(() => "?").join(", ");
  const values = cols.map((c) => row[c]);
  database.run(
    `INSERT OR IGNORE INTO ${table} (${cols.join(", ")}) VALUES (${placeholders})`,
    values
  );
}

/**
 * Update the row identified by uuid in the merged database with all fields
 * from the source row (except id).
 */
function updateRow(database, table, row) {
  const cols = Object.keys(row).filter((c) => c !== "id" && c !== "uuid");
  const setClauses = cols.map((c) => `${c} = ?`).join(", ");
  const values = [...cols.map((c) => row[c]), row.uuid];
  database.run(
    `UPDATE ${table} SET ${setClauses} WHERE uuid = ?`,
    values
  );
}

/**
 * Compare two row objects for equality, ignoring the `id` field (which is a
 * local auto-increment value and may legitimately differ between databases).
 */
function rowsEqual(a, b) {
  const keysA = Object.keys(a)
    .filter((k) => k !== "id")
    .sort();
  const keysB = Object.keys(b)
    .filter((k) => k !== "id")
    .sort();
  if (keysA.join() !== keysB.join()) return false;
  return keysA.every((k) => {
    // Treat null and undefined as equivalent.
    const va = a[k] ?? null;
    const vb = b[k] ?? null;
    return va === vb;
  });
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
  // Revoke the object URL after a short delay to free memory
  setTimeout(() => URL.revokeObjectURL(url), 10000);
}
