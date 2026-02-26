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

        console.log('[FileHandleStorage] Handle found in IndexedDB, checking permission state...');

        try {
            // Check current permission state
            const state = await handle.queryPermission({ mode: 'readwrite' });
            console.log('[FileHandleStorage] Current permission state:', state);
            
            // Return the handle regardless of state (granted or prompt).
            // If prompt, we'll handle the NotAllowedError later and show a "Grant" button.
            return handle;
        } catch (error) {
            // Handle is stale or invalid (e.g. file deleted or moved)
            console.warn('[FileHandleStorage] Handle validation failed:', error.name, error.message);
            await clearFileHandle();
            return null;
        }
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

        // Create empty file (write 0 bytes to initialize)
        const writable = await handle.createWritable();
        await writable.close();
        console.log('[FileHandleStorage] Empty file created successfully');

        // Request permission and store handle (reuse working pattern from open flow)
        // CRITICAL: Must be called during user gesture to show permission prompt
        const stored = await requestWritePermissionAndStore(handle);

        if (!stored) {
            throw new Error('Failed to request permission or store handle');
        }

        console.log('[FileHandleStorage] Permission granted and handle stored successfully');

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
