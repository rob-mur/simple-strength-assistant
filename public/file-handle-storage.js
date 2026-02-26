// IndexedDB helper for persisting File System Access API handles

const DB_NAME = 'workout-file-handles';
const DB_VERSION = 1;
const STORE_NAME = 'handles';
const HANDLE_KEY = 'workout-db-handle';

function openDB() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(DB_NAME, DB_VERSION);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);

        request.onupgradeneeded = (event) => {
            const db = event.target.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                db.createObjectStore(STORE_NAME);
            }
        };
    });
}

export async function storeFileHandle(handle) {
    try {
        const db = await openDB();
        const transaction = db.transaction(STORE_NAME, 'readwrite');
        const store = transaction.objectStore(STORE_NAME);

        await new Promise((resolve, reject) => {
            const request = store.put(handle, HANDLE_KEY);
            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });

        db.close();
        return true;
    } catch (error) {
        console.error('Failed to store file handle:', error);
        return false;
    }
}

// Request readwrite permission and store handle only if granted
// MUST be called during a user gesture (immediately after showOpenFilePicker)
export async function requestWritePermissionAndStore(handle) {
    try {
        console.log('[FileHandleStorage] Requesting readwrite permission...');

        // Request readwrite permission (must be done during user gesture)
        const permission = await handle.requestPermission({ mode: 'readwrite' });
        console.log('[FileHandleStorage] Permission result:', permission);

        if (permission === 'granted') {
            console.log('[FileHandleStorage] Write permission granted, storing handle...');
            // Now store the handle with full readwrite permission
            await storeFileHandle(handle);
            return true;
        } else {
            console.warn('[FileHandleStorage] Write permission not granted:', permission);
            return false;
        }
    } catch (error) {
        console.error('[FileHandleStorage] Failed to request write permission:', error);
        return false;
    }
}

export async function retrieveFileHandle() {
    try {
        console.log('[FileHandleStorage] Opening IndexedDB...');
        const db = await openDB();
        const transaction = db.transaction(STORE_NAME, 'readonly');
        const store = transaction.objectStore(STORE_NAME);

        const handle = await new Promise((resolve, reject) => {
            const request = store.get(HANDLE_KEY);
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });

        db.close();

        if (!handle) {
            console.log('[FileHandleStorage] No handle in IndexedDB');
            return null;
        }

        console.log('[FileHandleStorage] Handle found, checking permission state...');

        // CRITICAL: Check permission state before returning handle
        const options = { mode: 'readwrite' };
        const permission = await handle.queryPermission(options);
        console.log('[FileHandleStorage] Permission state:', permission);

        if (permission === 'granted') {
            console.log('[FileHandleStorage] Permission granted, handle ready to use');
            return handle;
        }

        if (permission === 'prompt') {
            // Permission expired or not yet granted
            // CRITICAL: requestPermission() REQUIRES a user gesture (button click, etc.)
            // During page load (auto-init), there is no user gesture context
            // Browsers will reject the call with an error or throw an exception
            // Solution: Clear the stale handle and force user to manually select file again
            console.warn('[FileHandleStorage] Permission state is prompt (requires user gesture)');
            console.log('[FileHandleStorage] Cannot auto-restore - user must select file again');
            await clearFileHandle();
            return null;
        }

        // permission === 'denied'
        console.warn('[FileHandleStorage] Permission permanently denied, clearing stale handle');
        await clearFileHandle();
        return null;

    } catch (error) {
        console.error('[FileHandleStorage] Error retrieving handle:', error);
        // Handle may be invalid (file deleted, drive disconnected, etc.)
        await clearFileHandle();
        return null;
    }
}

export async function clearFileHandle() {
    try {
        const db = await openDB();
        const transaction = db.transaction(STORE_NAME, 'readwrite');
        const store = transaction.objectStore(STORE_NAME);

        await new Promise((resolve, reject) => {
            const request = store.delete(HANDLE_KEY);
            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });

        db.close();
        return true;
    } catch (error) {
        console.error('Failed to clear file handle:', error);
        return false;
    }
}

export async function createNewDatabaseFile() {
    try {
        console.log('[FileHandleStorage] Opening save file picker for new database...');

        const options = {
            types: [{
                description: 'SQLite Database',
                accept: { 'application/x-sqlite3': ['.sqlite', '.db'] }
            }],
            suggestedName: 'workout-data.sqlite'
        };

        const handle = await window.showSaveFilePicker(options);
        console.log('[FileHandleStorage] File handle created for new database');

        // Request readwrite permission immediately (during gesture)
        const permission = await handle.requestPermission({ mode: 'readwrite' });
        console.log('[FileHandleStorage] Permission result:', permission);

        if (permission !== 'granted') {
            throw new Error('Permission not granted');
        }

        // Create empty file (write 0 bytes to initialize)
        const writable = await handle.createWritable();
        await writable.close();
        console.log('[FileHandleStorage] Empty file created successfully');

        // Store handle for persistence
        await storeFileHandle(handle);
        console.log('[FileHandleStorage] Handle stored in IndexedDB');

        return { success: true, handle };
    } catch (error) {
        console.error('[FileHandleStorage] Failed to create new database file:', error);
        return {
            success: false,
            error: error.name || 'Error',
            message: error.message || String(error)
        };
    }
}

window.fileHandleStorage = {
    storeFileHandle,
    retrieveFileHandle,
    clearFileHandle,
    requestWritePermissionAndStore,
    createNewDatabaseFile
};
