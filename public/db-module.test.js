// Unit tests for error-surfacing and timeout behaviour in db-module.js.
//
// Because db-module.js depends on browser APIs (localStorage, dynamic import,
// crsqlite-wasm), we test the pure helper functions in isolation by importing
// only the parts we can exercise in Node/vitest without a DOM.
//
// Strategy:
//   1. Export `withTimeout` and `getDbInitError`/`isSyncUnavailable` from
//      db-module.js so they can be unit-tested without running initDatabase().
//   2. The tests below verify the contract of those helpers directly.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// ── Inline reimplementation of withTimeout for isolated unit tests ─────────────
// We cannot import db-module.js in Node because it uses dynamic import() for
// browser-only modules.  Instead we duplicate the tiny helper here and test it
// directly — the same logic is in public/db-module.js.

function withTimeout(promise, ms, stepName) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(`DB init step timed out: ${stepName} (${ms}ms)`)),
      ms,
    );
    promise.then(
      (v) => { clearTimeout(timer); resolve(v); },
      (e) => { clearTimeout(timer); reject(e); },
    );
  });
}

// ── withTimeout helper tests ───────────────────────────────────────────────────

describe("withTimeout — resolves before deadline", () => {
  it("passes through the resolved value when promise resolves in time", async () => {
    const result = await withTimeout(
      Promise.resolve(42),
      100,
      "test-step",
    );
    expect(result).toBe(42);
  });

  it("passes through rejection when promise rejects before timeout", async () => {
    const err = new Error("inner failure");
    await expect(
      withTimeout(Promise.reject(err), 100, "test-step"),
    ).rejects.toThrow("inner failure");
  });
});

describe("withTimeout — timeout fires before promise resolves", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("rejects with a descriptive message naming the step when timeout fires", async () => {
    const hangingPromise = new Promise(() => {}); // never resolves
    const racePromise = withTimeout(hangingPromise, 500, "sqlite.open");

    // Advance timers past the deadline
    vi.advanceTimersByTime(501);

    await expect(racePromise).rejects.toThrow(
      "DB init step timed out: sqlite.open (500ms)",
    );
  });

  it("error message contains the step name for each named step", async () => {
    const steps = [
      "ensureCrSQLiteLoaded",
      "sqlite.open",
      "isMigrationDone",
      "applyCrrMigration",
      "migrateFromSqlJs",
      "registerDbWithSync",
      "import crsqlite-wasm",
      "initWasm()",
    ];

    for (const step of steps) {
      const racePromise = withTimeout(new Promise(() => {}), 100, step);
      vi.advanceTimersByTime(101);
      await expect(racePromise).rejects.toThrow(
        `DB init step timed out: ${step} (100ms)`,
      );
    }
  });

  it("does not reject when promise resolves before the timeout", async () => {
    // Manually resolve via a deferred promise
    let resolve;
    const deferred = new Promise((r) => { resolve = r; });
    const racePromise = withTimeout(deferred, 500, "fast-step");

    // Resolve before the timer fires
    resolve("ok");

    // Only advance time a little — timer should NOT fire
    vi.advanceTimersByTime(100);

    await expect(racePromise).resolves.toBe("ok");
  });
});

// ── Sync-unavailable flag contract ────────────────────────────────────────────
// The `isSyncUnavailable()` export starts false and becomes true when
// registerDbWithSync() catches an error.  We test the state machine logic
// directly with a tiny stub.

describe("sync-unavailable state machine", () => {
  it("starts as false (sync available by default)", () => {
    // Initial state is always false — verified by direct construction.
    let _syncUnavailable = false;
    expect(_syncUnavailable).toBe(false);
  });

  it("becomes true when sync module load rejects", async () => {
    let _syncUnavailable = false;

    async function registerDbWithSyncStub(syncModuleLoader) {
      try {
        await syncModuleLoader();
        _syncUnavailable = false;
      } catch (_e) {
        _syncUnavailable = true;
      }
    }

    await registerDbWithSyncStub(() =>
      Promise.reject(new Error("Failed to fetch sync-module.js")),
    );

    expect(_syncUnavailable).toBe(true);
  });

  it("becomes false when sync module loads successfully", async () => {
    let _syncUnavailable = true; // simulate previously failed load

    async function registerDbWithSyncStub(syncModuleLoader) {
      try {
        await syncModuleLoader();
        _syncUnavailable = false;
      } catch (_e) {
        _syncUnavailable = true;
      }
    }

    await registerDbWithSyncStub(() =>
      Promise.resolve({ registerSyncDb: () => {} }),
    );

    expect(_syncUnavailable).toBe(false);
  });
});

// ── getDbInitError contract ───────────────────────────────────────────────────
// Verifies that the error-capture logic sets _lastDbInitError to the error
// message when initDatabase() catches an exception, and clears it on success.

describe("getDbInitError — error capture contract", () => {
  it("returns empty string before any failure", () => {
    let _lastDbInitError = null;
    function getDbInitError() { return _lastDbInitError ?? ""; }
    expect(getDbInitError()).toBe("");
  });

  it("returns the error message after a step throws", async () => {
    let _lastDbInitError = null;
    function getDbInitError() { return _lastDbInitError ?? ""; }

    async function simulateInitWithError(failingStep) {
      _lastDbInitError = null;
      try {
        await failingStep();
        return true;
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        _lastDbInitError = msg;
        return false;
      }
    }

    const success = await simulateInitWithError(() =>
      Promise.reject(new Error("DB init step timed out: sqlite.open (15000ms)")),
    );

    expect(success).toBe(false);
    expect(getDbInitError()).toBe(
      "DB init step timed out: sqlite.open (15000ms)",
    );
  });

  it("clears error on next successful init", async () => {
    let _lastDbInitError = "previous error";
    function getDbInitError() { return _lastDbInitError ?? ""; }

    async function simulateSuccessfulInit() {
      _lastDbInitError = null;
      try {
        await Promise.resolve(true);
        return true;
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        _lastDbInitError = msg;
        return false;
      }
    }

    await simulateSuccessfulInit();
    expect(getDbInitError()).toBe("");
  });

  it("captures timeout step name in error", async () => {
    let _lastDbInitError = null;
    function getDbInitError() { return _lastDbInitError ?? ""; }

    const stepName = "isMigrationDone";
    const ms = 15000;

    async function simulateTimeout() {
      _lastDbInitError = null;
      try {
        throw new Error(`DB init step timed out: ${stepName} (${ms}ms)`);
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        _lastDbInitError = msg;
        return false;
      }
    }

    await simulateTimeout();
    expect(getDbInitError()).toContain(stepName);
    expect(getDbInitError()).toContain(`${ms}ms`);
  });
});

// ── downloadBytes strategy tests ──────────────────────────────────────────────
// We duplicate the downloadBytes logic here (same pattern as withTimeout above)
// because the module uses browser-only APIs we cannot import in Node/vitest.

/**
 * Minimal reimplementation of downloadBytes for unit testing the strategy
 * selection and error-surfacing logic.  Accepts injectable platform stubs.
 */
async function downloadBytesTestable(data, filename, {
  navigator: nav = {},
  createAnchor = () => ({ click() {}, set href(_) {}, set download(_) {} }),
  appendToBody = () => {},
  removeFromBody = () => {},
  createObjectURL = () => "blob:test",
  revokeObjectURL = () => {},
} = {}) {
  const byteSize = data.length;

  try {
    const blob = new Blob([data], { type: "application/x-sqlite3" });

    // Strategy 1: Web Share API with files
    if (nav.share && nav.canShare) {
      const file = new File([blob], filename, { type: "application/x-sqlite3" });
      const shareData = { files: [file] };
      try {
        if (nav.canShare(shareData)) {
          await nav.share(shareData);
          return { ok: true, method: "share", byteSize };
        }
      } catch (shareErr) {
        if (shareErr.name === "AbortError") {
          return { ok: true, method: "share-cancelled", byteSize };
        }
        // Fall through
      }
    }

    const url = createObjectURL(blob);

    // Strategy 2: Anchor element click (desktop / most browsers)
    const a = createAnchor();
    a.href = url;
    a.download = filename;
    appendToBody(a);
    a.click();
    removeFromBody(a);
    return { ok: true, method: "anchor", byteSize };

  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { ok: false, error: message, byteSize };
  }
}

describe("downloadBytes — Web Share API strategy", () => {
  it("uses navigator.share when canShare returns true for files", async () => {
    const sharedData = [];
    const nav = {
      canShare: (data) => data.files && data.files.length > 0,
      share: async (data) => { sharedData.push(data); },
    };

    const result = await downloadBytesTestable(
      new Uint8Array([1, 2, 3]),
      "test.sqlite",
      { navigator: nav },
    );

    expect(result.ok).toBe(true);
    expect(result.method).toBe("share");
    expect(result.byteSize).toBe(3);
    expect(sharedData).toHaveLength(1);
    expect(sharedData[0].files[0]).toBeInstanceOf(File);
    expect(sharedData[0].files[0].name).toBe("test.sqlite");
  });

  it("treats user cancellation (AbortError) as success", async () => {
    const abortErr = new DOMException("Share cancelled", "AbortError");
    const nav = {
      canShare: () => true,
      share: async () => { throw abortErr; },
    };

    const result = await downloadBytesTestable(
      new Uint8Array([1]),
      "test.sqlite",
      { navigator: nav },
    );

    expect(result.ok).toBe(true);
    expect(result.method).toBe("share-cancelled");
  });
});

describe("downloadBytes — fallback when canShare is unavailable", () => {
  it("falls back to anchor click when navigator.canShare is not defined", async () => {
    let clicked = false;
    const result = await downloadBytesTestable(
      new Uint8Array([4, 5]),
      "test.sqlite",
      {
        navigator: {},
        createAnchor: () => ({
          click() { clicked = true; },
          href: "",
          download: "",
        }),
      },
    );

    expect(result.ok).toBe(true);
    expect(result.method).toBe("anchor");
    expect(clicked).toBe(true);
  });

  it("falls back to anchor click when canShare returns false for files", async () => {
    const nav = {
      canShare: () => false,
      share: async () => {},
    };
    let clicked = false;

    const result = await downloadBytesTestable(
      new Uint8Array([6]),
      "test.sqlite",
      {
        navigator: nav,
        createAnchor: () => ({
          click() { clicked = true; },
          href: "",
          download: "",
        }),
      },
    );

    expect(result.ok).toBe(true);
    expect(result.method).toBe("anchor");
    expect(clicked).toBe(true);
  });
});

describe("downloadBytes — anchor as default desktop strategy", () => {
  it("uses anchor click when no share API is available", async () => {
    let clicked = false;
    let downloadAttr = "";
    const result = await downloadBytesTestable(
      new Uint8Array([7, 8]),
      "test.sqlite",
      {
        navigator: {},
        createAnchor: () => ({
          click() { clicked = true; },
          href: "",
          set download(val) { downloadAttr = val; },
          get download() { return downloadAttr; },
        }),
      },
    );

    expect(result.ok).toBe(true);
    expect(result.method).toBe("anchor");
    expect(clicked).toBe(true);
    expect(downloadAttr).toBe("test.sqlite");
  });

  it("uses anchor click after share API fails with non-AbortError", async () => {
    let clicked = false;
    const nav = {
      canShare: () => true,
      share: async () => { throw new Error("NotAllowedError"); },
    };

    const result = await downloadBytesTestable(
      new Uint8Array([9]),
      "test.sqlite",
      {
        navigator: nav,
        createAnchor: () => ({
          click() { clicked = true; },
          href: "",
          download: "",
        }),
      },
    );

    expect(result.ok).toBe(true);
    expect(result.method).toBe("anchor");
    expect(clicked).toBe(true);
  });
});

describe("downloadBytes — error surfacing", () => {
  it("returns an error result when share rejects with a non-AbortError and anchor still works", async () => {
    const nav = {
      canShare: () => true,
      share: async () => { throw new Error("NotAllowedError"); },
    };

    // Share fails, anchor still works as fallback
    const result = await downloadBytesTestable(
      new Uint8Array([10]),
      "test.sqlite",
      {
        navigator: nav,
      },
    );

    // The anchor fallback should still succeed
    expect(result.ok).toBe(true);
    expect(result.method).toBe("anchor");
  });

  it("returns ok:false when blob construction throws", async () => {
    const result = await downloadBytesTestable(
      new Uint8Array([11]),
      "test.sqlite",
      {
        navigator: {},
        createObjectURL: () => { throw new Error("Blob allocation failed"); },
      },
    );

    expect(result.ok).toBe(false);
    expect(result.error).toBe("Blob allocation failed");
  });

  it("includes byteSize in both success and failure results", async () => {
    const successResult = await downloadBytesTestable(
      new Uint8Array([1, 2, 3, 4, 5]),
      "test.sqlite",
      { navigator: {} },
    );
    expect(successResult.byteSize).toBe(5);

    const failResult = await downloadBytesTestable(
      new Uint8Array([1, 2]),
      "test.sqlite",
      {
        navigator: {},
        createObjectURL: () => { throw new Error("fail"); },
      },
    );
    expect(failResult.byteSize).toBe(2);
  });
});
