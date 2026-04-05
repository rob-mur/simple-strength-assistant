// OPFS (Origin Private File System) storage backend.
// Replaces the File System Access API + IndexedDB approach, making the app
// functional on iOS Safari 16.4+ and Chrome Android without user gestures.
//
// On iOS Safari below 16.4, isOPFSAvailable() returns false and the app loads
// but data does not persist (same as the previous iOS behaviour).

const OPFS_FILENAME = "workout-data.sqlite";

/**
 * Returns true if OPFS is available in this browser.
 * iOS Safari 16.4+ and Chrome Android both support it.
 */
function isOPFSAvailable() {
  return (
    typeof navigator !== "undefined" &&
    typeof navigator.storage !== "undefined" &&
    typeof navigator.storage.getDirectory === "function"
  );
}

/**
 * Opens (or optionally creates) the OPFS file handle for the workout database.
 * Returns a FileSystemFileHandle on success, or null if OPFS is unavailable or
 * the file does not exist yet (when create=false).
 */
async function getOPFSFileHandle(create = false) {
  if (!isOPFSAvailable()) {
    console.log("[OPFS] OPFS not available in this browser");
    return null;
  }

  try {
    const root = await navigator.storage.getDirectory();
    const fileHandle = await root.getFileHandle(OPFS_FILENAME, { create });
    return fileHandle;
  } catch (error) {
    if (error.name === "NotFoundError") {
      // File does not exist yet and create=false — expected on first run
      return null;
    }
    console.error("[OPFS] Failed to get OPFS file handle:", error);
    return null;
  }
}

/**
 * No-op: OPFS handles are always re-obtainable from the file system on demand;
 * there is nothing to persist. Kept for interface compatibility with db-module.js.
 */
export async function storeFileHandle(_handle) {
  return true;
}

/**
 * Retrieves the OPFS file handle for the workout database.
 * Returns null if the file does not exist yet or OPFS is unavailable.
 * The returned FileSystemFileHandle supports getFile() and createWritable(),
 * matching the interface previously provided by the File System Access API.
 */
export async function retrieveFileHandle() {
  console.log("[OPFS] Retrieving OPFS file handle...");
  const handle = await getOPFSFileHandle(false);

  if (!handle) {
    console.log("[OPFS] No existing OPFS database file found");
    return null;
  }

  console.log("[OPFS] OPFS file handle retrieved successfully");
  return handle;
}

/**
 * Removes the OPFS workout database file so the app starts fresh.
 */
export async function clearFileHandle() {
  if (!isOPFSAvailable()) {
    return true;
  }

  try {
    const root = await navigator.storage.getDirectory();
    await root.removeEntry(OPFS_FILENAME);
    console.log("[OPFS] Cleared OPFS database file");
    return true;
  } catch (error) {
    if (error.name === "NotFoundError") {
      // File did not exist — that is fine
      return true;
    }
    console.error("[OPFS] Failed to clear OPFS file:", error);
    return false;
  }
}

/**
 * No-op: OPFS files require no explicit permission grants.
 * OPFS storage is always readable and writable without user gestures.
 * Returns true to signal success, keeping the interface compatible.
 */
export async function requestWritePermissionAndStore(_handle) {
  return true;
}

/**
 * Creates a new (empty) OPFS database file and returns a handle to it.
 * Returns { success: true, handle } on success,
 * or { success: false, error, message } on failure.
 */
export async function createNewDatabaseFile() {
  if (!isOPFSAvailable()) {
    return {
      success: false,
      error: "NotSupportedError",
      message: "OPFS is not available in this browser (iOS Safari < 16.4)",
    };
  }

  try {
    console.log("[OPFS] Creating new OPFS database file...");
    const handle = await getOPFSFileHandle(true);

    if (!handle) {
      return {
        success: false,
        error: "Error",
        message: "Failed to obtain OPFS file handle",
      };
    }

    // Truncate to zero bytes so it is guaranteed to start empty
    const writable = await handle.createWritable();
    await writable.close();

    console.log("[OPFS] New OPFS database file created successfully");
    return { success: true, handle };
  } catch (error) {
    console.error("[OPFS] Failed to create new database file:", error);
    return {
      success: false,
      error: error.name || "Error",
      message: error.message || String(error),
    };
  }
}

window.fileHandleStorage = {
  storeFileHandle,
  retrieveFileHandle,
  clearFileHandle,
  requestWritePermissionAndStore,
  createNewDatabaseFile,
};
