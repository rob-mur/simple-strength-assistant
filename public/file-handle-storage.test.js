// Unit tests for the OPFS storage backend in file-handle-storage.js
// These tests run in a Node/vitest environment with a mocked OPFS API.

import { describe, it, expect, beforeEach, vi } from "vitest";

// ── OPFS mock ──────────────────────────────────────────────────────────────────
// Provides a minimal in-memory implementation of the OPFS subset used by
// file-handle-storage.js:
//   navigator.storage.getDirectory()
//   FileSystemDirectoryHandle.getFileHandle(name, { create })
//   FileSystemDirectoryHandle.removeEntry(name)
//   FileSystemFileHandle.getFile()
//   FileSystemFileHandle.createWritable()
//   WritableStream.write(data) / close()
//   File.arrayBuffer()

function createMockOPFS() {
  const files = new Map(); // filename → Uint8Array

  function makeWritable(name) {
    const chunks = [];
    return {
      async write(data) {
        if (data instanceof Uint8Array) {
          chunks.push(data);
        } else if (data instanceof ArrayBuffer) {
          chunks.push(new Uint8Array(data));
        }
        // zero-length write (from writable.close() only) is fine
      },
      async close() {
        const total = chunks.reduce((acc, c) => acc + c.byteLength, 0);
        const result = new Uint8Array(total);
        let offset = 0;
        for (const chunk of chunks) {
          result.set(chunk, offset);
          offset += chunk.byteLength;
        }
        files.set(name, result);
      },
    };
  }

  function makeFileHandle(name) {
    return {
      kind: "file",
      name,
      async getFile() {
        const data = files.get(name) ?? new Uint8Array(0);
        return {
          size: data.byteLength,
          async arrayBuffer() {
            // Return a copy so mutations don't affect stored data
            return data.buffer.slice(
              data.byteOffset,
              data.byteOffset + data.byteLength,
            );
          },
        };
      },
      async createWritable() {
        return makeWritable(name);
      },
      // OPFS handles don't need permission — no-op for queryPermission
      async queryPermission() {
        return "granted";
      },
    };
  }

  function makeDirHandle() {
    return {
      async getFileHandle(name, options = {}) {
        if (!options.create && !files.has(name)) {
          throw new DOMException(`File not found: ${name}`, "NotFoundError");
        }
        return makeFileHandle(name);
      },
      async removeEntry(name) {
        if (!files.has(name)) {
          throw new DOMException(`File not found: ${name}`, "NotFoundError");
        }
        files.delete(name);
      },
    };
  }

  return {
    storage: {
      async getDirectory() {
        return makeDirHandle();
      },
    },
    files, // expose for assertions
  };
}

// ── Module loader ──────────────────────────────────────────────────────────────

async function loadModule(mockOPFS) {
  // Patch navigator.storage with the mock OPFS implementation.
  // Use Object.defineProperty because `navigator` may be read-only in Node.
  Object.defineProperty(global, "navigator", {
    value: { storage: mockOPFS.storage },
    writable: true,
    configurable: true,
  });

  // Also provide a stub `window` so the guard at the bottom of the module
  // does not throw a ReferenceError.
  if (typeof global.window === "undefined") {
    global.window = {};
  }

  const mod = await import("./file-handle-storage.js");
  return mod;
}

// ── Test suite ─────────────────────────────────────────────────────────────────

describe("OPFS storage backend — save / load / clear", () => {
  let mock;
  let createNewDatabaseFile;
  let retrieveFileHandle;
  let clearFileHandle;

  beforeEach(async () => {
    vi.resetModules();
    mock = createMockOPFS();
    const mod = await loadModule(mock);
    createNewDatabaseFile = mod.createNewDatabaseFile;
    retrieveFileHandle = mod.retrieveFileHandle;
    clearFileHandle = mod.clearFileHandle;
  });

  // ── Tracer bullet: save → load returns same bytes ────────────────────────────

  it("save then load returns the same bytes", async () => {
    // Create / open the OPFS file (acts as "save" for an initially empty db)
    const result = await createNewDatabaseFile();
    expect(result.success).toBe(true);

    const handle = result.handle;
    expect(handle).toBeTruthy();

    // Write some bytes through the handle
    const data = new Uint8Array([83, 81, 76, 105, 116, 101]); // "SQLite"
    const writable = await handle.createWritable();
    await writable.write(data);
    await writable.close();

    // Load via retrieveFileHandle and read back
    const loaded = await retrieveFileHandle();
    expect(loaded).toBeTruthy();

    const file = await loaded.getFile();
    const buf = await file.arrayBuffer();
    expect(new Uint8Array(buf)).toEqual(data);
  });

  // ── Load before any save returns empty ───────────────────────────────────────

  it("load before any save returns null (no file exists yet)", async () => {
    const handle = await retrieveFileHandle();
    // OPFS has no existing file — should return null
    expect(handle).toBeNull();
  });

  // ── Save twice then load returns the second value ────────────────────────────

  it("saving twice then loading returns the second saved value", async () => {
    const first = new Uint8Array([1, 2, 3]);
    const second = new Uint8Array([4, 5, 6, 7]);

    // First save
    const result = await createNewDatabaseFile();
    const handle = result.handle;

    const w1 = await handle.createWritable();
    await w1.write(first);
    await w1.close();

    // Second save (overwrite)
    const w2 = await handle.createWritable();
    await w2.write(second);
    await w2.close();

    // Load and verify second value is present
    const loaded = await retrieveFileHandle();
    const file = await loaded.getFile();
    const buf = await file.arrayBuffer();
    expect(new Uint8Array(buf)).toEqual(second);
  });

  // ── Clear removes the file ────────────────────────────────────────────────────

  it("clear removes stored data so subsequent load returns null", async () => {
    const result = await createNewDatabaseFile();
    const handle = result.handle;

    const w = await handle.createWritable();
    await w.write(new Uint8Array([10, 20, 30]));
    await w.close();

    // Confirm data is there
    expect(await retrieveFileHandle()).toBeTruthy();

    // Clear it
    await clearFileHandle();

    // Now load should return null
    expect(await retrieveFileHandle()).toBeNull();
  });
});

describe("openExistingDatabaseFile — open without truncation", () => {
  let mock;
  let createNewDatabaseFile;
  let openExistingDatabaseFile;
  let retrieveFileHandle;

  beforeEach(async () => {
    vi.resetModules();
    mock = createMockOPFS();
    const mod = await loadModule(mock);
    createNewDatabaseFile = mod.createNewDatabaseFile;
    openExistingDatabaseFile = mod.openExistingDatabaseFile;
    retrieveFileHandle = mod.retrieveFileHandle;
  });

  // ── First run: creates an empty file ────────────────────────────────────────

  it("returns a handle even when no file exists yet (first run)", async () => {
    const result = await openExistingDatabaseFile();
    expect(result.success).toBe(true);
    expect(result.handle).toBeTruthy();
  });

  // ── Does not internally truncate the file on open ────────────────────────────
  // The key invariant: calling openExistingDatabaseFile() on a file that
  // already has content must not erase that content.  We seed data via the
  // underlying mock's file map directly to simulate an already-populated OPFS
  // file, then assert the handle still exposes that data after the open.

  it("does not truncate an existing file — data is preserved across open calls", async () => {
    const originalData = new Uint8Array([83, 81, 76, 105, 116, 101]); // "SQLite"

    // Seed the mock's file store directly (simulates a populated OPFS file
    // left by a previous session — no writable stream involved).
    mock.files.set("workout-data.sqlite", originalData);

    // Opening must not truncate — the handle should see the existing data.
    const result = await openExistingDatabaseFile();
    expect(result.success).toBe(true);

    const file = await result.handle.getFile();
    const buf = await file.arrayBuffer();
    expect(new Uint8Array(buf)).toEqual(originalData);
  });

  // ── Contrast: createNewDatabaseFile DOES truncate ────────────────────────────
  // This documents the intentional difference between the two functions.

  it("createNewDatabaseFile truncates while openExistingDatabaseFile does not", async () => {
    const originalData = new Uint8Array([1, 2, 3, 4, 5]);
    mock.files.set("workout-data.sqlite", originalData);

    // createNewDatabaseFile should wipe the file
    const freshResult = await createNewDatabaseFile();
    expect(freshResult.success).toBe(true);
    const freshFile = await freshResult.handle.getFile();
    const freshBuf = await freshFile.arrayBuffer();
    expect(new Uint8Array(freshBuf).byteLength).toBe(0); // truncated

    // Re-seed and verify openExistingDatabaseFile does NOT truncate
    mock.files.set("workout-data.sqlite", originalData);
    const openResult = await openExistingDatabaseFile();
    expect(openResult.success).toBe(true);
    const openFile = await openResult.handle.getFile();
    const openBuf = await openFile.arrayBuffer();
    expect(new Uint8Array(openBuf)).toEqual(originalData); // preserved
  });

  // ── Existing file is accessible via retrieveFileHandle after open ────────────

  it("after opening a seeded file, the file is retrievable via retrieveFileHandle", async () => {
    // Seed the mock so the file already exists (simulates a prior session)
    mock.files.set("workout-data.sqlite", new Uint8Array([1, 2, 3]));

    await openExistingDatabaseFile();

    const retrieved = await retrieveFileHandle();
    expect(retrieved).toBeTruthy();
  });
});

describe("storeFileHandle / requestWritePermissionAndStore (no-op compatibility shims)", () => {
  let mock;
  let storeFileHandle;
  let requestWritePermissionAndStore;

  beforeEach(async () => {
    vi.resetModules();
    mock = createMockOPFS();
    const mod = await loadModule(mock);
    storeFileHandle = mod.storeFileHandle;
    requestWritePermissionAndStore = mod.requestWritePermissionAndStore;
  });

  it("storeFileHandle always returns true (no-op)", async () => {
    expect(await storeFileHandle({})).toBe(true);
  });

  it("requestWritePermissionAndStore always returns true (no-op)", async () => {
    expect(await requestWritePermissionAndStore({})).toBe(true);
  });
});
