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
            // Try to request permission - this may auto-grant if user previously selected "Remember"
            // or it may require a user gesture (in which case it will fail or return 'prompt' again)
            console.log('[FileHandleStorage] Permission state is prompt, attempting to request permission...');

            try {
                const requestedPermission = await handle.requestPermission(options);
                console.log('[FileHandleStorage] Permission request result:', requestedPermission);

                if (requestedPermission === 'granted') {
                    console.log('[FileHandleStorage] Permission granted after request');
                    return handle;
                } else if (requestedPermission === 'prompt') {
                    // Permission still not granted - needs user gesture
                    console.warn('[FileHandleStorage] Permission still requires user action - clearing handle');
                    await clearFileHandle();
                    return null;
                } else {
                    // 'denied'
                    console.warn('[FileHandleStorage] Permission denied by user');
                    await clearFileHandle();
                    return null;
                }
            } catch (error) {
                console.error('[FileHandleStorage] requestPermission failed:', error);
                // Clear stale handle since we can't use it
                await clearFileHandle();
                return null;
            }
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

window.fileHandleStorage = {
    storeFileHandle,
    retrieveFileHandle,
    clearFileHandle
};
