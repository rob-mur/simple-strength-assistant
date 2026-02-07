let db = null;
let SQL = null;

async function ensureSQLLoaded() {
    if (SQL) return SQL;

    if (typeof window.initSqlJs === 'undefined') {
        throw new Error('sql.js not loaded. Make sure to include it in your HTML.');
    }

    SQL = await window.initSqlJs({
        locateFile: file => `public/${file}`
    });

    return SQL;
}

export async function initDatabase(fileData) {
    try {
        await ensureSQLLoaded();

        if (fileData && fileData.length > 0) {
            const uint8Array = new Uint8Array(fileData);
            db = new SQL.Database(uint8Array);
            console.log('Database loaded from file data');
        } else {
            db = new SQL.Database();
            console.log('New database created');
        }

        return true;
    } catch (error) {
        console.error('Failed to initialize database:', error);
        return false;
    }
}

export async function executeQuery(sql, params) {
    if (!db) {
        throw new Error('Database not initialized');
    }

    try {
        const results = db.exec(sql, params);

        if (results.length === 0) {
            return null;
        }

        const result = results[0];
        const columns = result.columns;
        const values = result.values;

        if (sql.trim().toUpperCase().startsWith('SELECT') || sql.includes('RETURNING')) {
            if (values.length === 0) {
                return [];
            }

            const rows = values.map(row => {
                const obj = {};
                columns.forEach((col, idx) => {
                    obj[col] = row[idx];
                });
                return obj;
            });

            if (sql.includes('RETURNING')) {
                return rows[0];
            }

            return rows;
        }

        return { changes: db.getRowsModified() };
    } catch (error) {
        console.error('Query execution failed:', error.message || error);
        throw error;
    }
}

export async function executeTransaction(queries) {
    if (!db) {
        throw new Error('Database not initialized');
    }

    try {
        db.run('BEGIN TRANSACTION');

        const results = [];
        for (const { sql, params } of queries) {
            // Use db.run() for proper transaction support instead of db.exec()
            // db.exec() doesn't respect transaction boundaries properly in sql.js
            const stmt = db.prepare(sql);
            stmt.bind(params || []);

            const result = [];
            while (stmt.step()) {
                const row = stmt.getAsObject();
                result.push(row);
            }
            stmt.free();

            results.push(result.length > 0 ? result : null);
        }

        db.run('COMMIT');
        return results;
    } catch (error) {
        // Attempt to rollback, but preserve the original error
        try {
            db.run('ROLLBACK');
        } catch (rollbackError) {
            console.error('ROLLBACK failed:', rollbackError);
            // Create a combined error message
            const combinedError = new Error(
                `Transaction failed: ${error.message}. Additionally, ROLLBACK failed: ${rollbackError.message}`
            );
            combinedError.originalError = error;
            combinedError.rollbackError = rollbackError;
            throw combinedError;
        }
        console.error('Transaction failed:', error);
        throw error;
    }
}

export async function exportDatabase() {
    if (!db) {
        throw new Error('Database not initialized');
    }

    try {
        const uint8Array = db.export();
        return uint8Array;
    } catch (error) {
        console.error('Failed to export database:', error);
        throw error;
    }
}

window.dbBridge = {
    initDatabase,
    executeQuery,
    executeTransaction,
    exportDatabase
};
