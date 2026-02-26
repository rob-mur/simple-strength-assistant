let db = null;
let SQL = null;

async function ensureSQLLoaded() {
    if (SQL) return SQL;

    if (typeof window.initSqlJs === 'undefined') {
        throw new Error('sql.js not loaded. Make sure to include it in your HTML.');
    }

    SQL = await window.initSqlJs({
        locateFile: file => file
    });

    return SQL;
}

export async function initDatabase(fileData) {
    try {
        await ensureSQLLoaded();

        if (fileData && fileData.length > 0) {
            const uint8Array = new Uint8Array(fileData);
            db = new SQL.Database(uint8Array);
        } else {
            db = new SQL.Database();
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
            try { stmt.free(); } catch(e) {}
        }
        console.error('Query execution failed:', error.message || error);
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