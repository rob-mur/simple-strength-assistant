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

export async function retrieveFileHandle() {
    try {
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
            return null;
        }

        // Verify we still have permission to access the handle
        const options = { mode: 'readwrite' };
        const permission = await handle.queryPermission(options);

        if (permission === 'granted') {
            return handle;
        }

        // Try to request permission
        const requestedPermission = await handle.requestPermission(options);
        if (requestedPermission === 'granted') {
            return handle;
        }

        // Permission denied, remove the stale handle
        await clearFileHandle();
        return null;
    } catch (error) {
        console.error('Failed to retrieve file handle:', error);
        // If there's an error (e.g., handle no longer valid), clear it
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

window.fileHandleStorage = {
    storeFileHandle,
    retrieveFileHandle,
    clearFileHandle
};
