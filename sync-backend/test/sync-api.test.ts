import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  test,
} from "bun:test";
import {
  access,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createApp } from "../src/app.ts";

async function fileExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function startServer(dataDir: string) {
  const app = createApp(dataDir);
  const server = Bun.serve({ fetch: app.fetch, port: 0 });
  const baseUrl = `http://localhost:${server.port}`;
  return { server, baseUrl };
}

describe("sync API", () => {
  let dataDir: string;
  let server: ReturnType<typeof Bun.serve>;
  let baseUrl: string;

  beforeAll(async () => {
    dataDir = await mkdtemp(join(tmpdir(), "sync-test-"));
    ({ server, baseUrl } = startServer(dataDir));
  });

  afterAll(async () => {
    server.stop(true);
    await rm(dataDir, { recursive: true, force: true });
  });

  test("GET returns 404 if the slot has never been written to", async () => {
    const res = await fetch(`${baseUrl}/sync/nonexistent`);
    expect(res.status).toBe(404);
  });

  test("POST stores blob and returns 200, GET returns same bytes", async () => {
    const blob = new Uint8Array([0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03]);
    const clock = { nodeA: 1 };

    const postRes = await fetch(`${baseUrl}/sync/test-slot`, {
      method: "POST",
      body: blob,
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify(clock),
      },
    });
    expect(postRes.status).toBe(200);

    const getRes = await fetch(`${baseUrl}/sync/test-slot`);
    expect(getRes.status).toBe(200);

    const returned = new Uint8Array(await getRes.arrayBuffer());
    expect(returned).toEqual(blob);
  });

  test("POST writes clock.json with valid JSON matching the pushed vector clock", async () => {
    const clock = { nodeA: 2, nodeB: 1 };

    await fetch(`${baseUrl}/sync/clock-test`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify(clock),
      },
    });

    const clockJson = JSON.parse(
      await readFile(join(dataDir, "clock-test", "clock.json"), "utf-8"),
    );
    expect(clockJson).toEqual(clock);
  });

  test("POST writes meta.json with last_modified and blob_size", async () => {
    const blob = new Uint8Array([10, 20, 30, 40, 50]);
    const before = Date.now();

    await fetch(`${baseUrl}/sync/meta-test`, {
      method: "POST",
      body: blob,
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify({ n: 1 }),
      },
    });

    const after = Date.now();
    const meta = JSON.parse(
      await readFile(join(dataDir, "meta-test", "meta.json"), "utf-8"),
    );
    expect(meta.blob_size).toBe(5);
    expect(meta.last_modified).toBeGreaterThanOrEqual(before);
    expect(meta.last_modified).toBeLessThanOrEqual(after);
  });

  test("second POST overwrites blob and updates clock.json and meta.json", async () => {
    const syncId = "overwrite-test";

    // First push
    await fetch(`${baseUrl}/sync/${syncId}`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify({ n: 1 }),
      },
    });

    // Second push with different data
    const newBlob = new Uint8Array([4, 5, 6, 7]);
    const newClock = { n: 2 };

    const postRes = await fetch(`${baseUrl}/sync/${syncId}`, {
      method: "POST",
      body: newBlob,
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify(newClock),
      },
    });
    expect(postRes.status).toBe(200);

    // Verify blob was overwritten
    const getRes = await fetch(`${baseUrl}/sync/${syncId}`);
    const returned = new Uint8Array(await getRes.arrayBuffer());
    expect(returned).toEqual(newBlob);

    // Verify clock.json updated
    const clockJson = JSON.parse(
      await readFile(join(dataDir, syncId, "clock.json"), "utf-8"),
    );
    expect(clockJson).toEqual(newClock);

    // Verify meta.json updated
    const meta = JSON.parse(
      await readFile(join(dataDir, syncId, "meta.json"), "utf-8"),
    );
    expect(meta.blob_size).toBe(4);
  });

  test("POST returns 400 for invalid sync_id", async () => {
    const res = await fetch(`${baseUrl}/sync/has%2Fslash`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify({ n: 1 }),
      },
    });
    expect(res.status).toBe(400);
  });

  test("GET returns 400 for invalid sync_id", async () => {
    const res = await fetch(`${baseUrl}/sync/bad%2Fid`);
    expect(res.status).toBe(400);
  });

  test("POST returns 400 for malformed X-Vector-Clock header", async () => {
    const res = await fetch(`${baseUrl}/sync/clock-bad`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": "not-json{{{",
      },
    });
    expect(res.status).toBe(400);
  });

  test("POST returns 400 for structurally invalid X-Vector-Clock", async () => {
    const res = await fetch(`${baseUrl}/sync/clock-struct`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify([1, 2, 3]),
      },
    });
    expect(res.status).toBe(400);

    const res2 = await fetch(`${baseUrl}/sync/clock-struct2`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify({ node: "not-a-number" }),
      },
    });
    expect(res2.status).toBe(400);
  });

  test("POST returns 413 for oversized Content-Length", async () => {
    // Use a raw HTTP request so Content-Length isn't overridden by fetch()
    const url = new URL(`${baseUrl}/sync/too-large`);
    const socket = await Bun.connect({
      hostname: url.hostname,
      port: Number(url.port),
      socket: {
        data(_socket, data) {
          socket.data += new TextDecoder().decode(data);
        },
        open() {},
        close() {},
        error() {},
      },
      data: "",
    });

    const body = new Uint8Array([1]);
    const request = [
      `POST /sync/too-large HTTP/1.1`,
      `Host: ${url.hostname}:${url.port}`,
      `Content-Type: application/octet-stream`,
      `Content-Length: ${100 * 1024 * 1024}`,
      `X-Vector-Clock: ${JSON.stringify({ n: 1 })}`,
      ``,
      ``,
    ].join("\r\n");

    socket.write(request);
    socket.write(body);

    // Wait for response
    await Bun.sleep(200);
    socket.end();

    expect(socket.data).toContain("413");
  });

  test("GET /sync/:sync_id/metadata returns 404 for a slot that has never been pushed", async () => {
    const res = await fetch(`${baseUrl}/sync/no-such-slot/metadata`);
    expect(res.status).toBe(404);
  });

  test("GET /sync/:sync_id/metadata returns correct JSON shape after push", async () => {
    const blob = new Uint8Array([0xaa, 0xbb, 0xcc, 0xdd]);
    const clock = { device_a: 3 };
    const before = Date.now();

    await fetch(`${baseUrl}/sync/metadata-test`, {
      method: "POST",
      body: blob,
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify(clock),
      },
    });

    const after = Date.now();

    const res = await fetch(`${baseUrl}/sync/metadata-test/metadata`);
    expect(res.status).toBe(200);

    const json = await res.json();

    // vector_clock matches what was pushed
    expect(json.vector_clock).toEqual(clock);

    // blob_size matches byte length
    expect(json.blob_size).toBe(4);

    // last_modified is a Unix-millisecond timestamp within the push window
    expect(json.last_modified).toBeGreaterThanOrEqual(before);
    expect(json.last_modified).toBeLessThanOrEqual(after);

    // conflicted is false for non-conflicted slot
    expect(json.conflicted).toBe(false);
  });

  test("GET /sync/:sync_id/metadata returns 500 for corrupt clock.json", async () => {
    const slotDir = join(dataDir, "corrupt-clock");
    await mkdir(slotDir, { recursive: true });
    await writeFile(join(slotDir, "clock.json"), "not-valid-json");
    await writeFile(
      join(slotDir, "meta.json"),
      JSON.stringify({ blob_size: 1, last_modified: 0 }),
    );

    const res = await fetch(`${baseUrl}/sync/corrupt-clock/metadata`);
    expect(res.status).toBe(500);
  });

  test("separate data directories are isolated from each other", async () => {
    const customDir = await mkdtemp(join(tmpdir(), "sync-custom-"));
    const customServer = startServer(customDir);

    const blob = new Uint8Array([0xca, 0xfe]);
    await fetch(`${customServer.baseUrl}/sync/env-test`, {
      method: "POST",
      body: blob,
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify({ x: 1 }),
      },
    });

    const getRes = await fetch(`${customServer.baseUrl}/sync/env-test`);
    expect(getRes.status).toBe(200);
    const returned = new Uint8Array(await getRes.arrayBuffer());
    expect(returned).toEqual(blob);

    // Verify blob does NOT exist in the original dataDir
    const crossRes = await fetch(`${baseUrl}/sync/env-test`);
    expect(crossRes.status).toBe(404);

    customServer.server.stop(true);
    await rm(customDir, { recursive: true, force: true });
  });
});

describe("conflict detection", () => {
  let dataDir: string;
  let server: ReturnType<typeof Bun.serve>;
  let baseUrl: string;

  beforeEach(async () => {
    dataDir = await mkdtemp(join(tmpdir(), "sync-conflict-"));
    const app = createApp(dataDir);
    server = Bun.serve({ fetch: app.fetch, port: 0 });
    baseUrl = `http://localhost:${server.port}`;
  });

  afterEach(async () => {
    server.stop(true);
    await rm(dataDir, { recursive: true, force: true });
  });

  async function push(
    syncId: string,
    blob: Uint8Array,
    clock: Record<string, number>,
  ) {
    return fetch(`${baseUrl}/sync/${syncId}`, {
      method: "POST",
      body: blob,
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Vector-Clock": JSON.stringify(clock),
      },
    });
  }

  test("fast-forward push updates canonical blob, no conflict created", async () => {
    await push("ff-test", new Uint8Array([1]), { a: 1 });
    const res = await push("ff-test", new Uint8Array([2]), { a: 2 });
    expect(res.status).toBe(200);

    // No conflicts directory
    const conflictsDir = join(dataDir, "ff-test", "conflicts");
    expect(await fileExists(conflictsDir)).toBe(false);

    // Blob updated
    const getRes = await fetch(`${baseUrl}/sync/ff-test`);
    expect(getRes.status).toBe(200);
    const returned = new Uint8Array(await getRes.arrayBuffer());
    expect(returned).toEqual(new Uint8Array([2]));
  });

  test("diverging push moves canonical blob to conflicts and stores incoming as second entry", async () => {
    // Initial push from device A
    await push("conflict-test", new Uint8Array([0xaa]), { a: 1 });

    // Diverging push from device B (concurrent clock)
    const res = await push("conflict-test", new Uint8Array([0xbb]), { b: 1 });
    expect(res.status).toBe(200);

    // Top-level blob should be removed
    expect(await fileExists(join(dataDir, "conflict-test", "blob"))).toBe(
      false,
    );

    // conflicts/ directory should have two entries
    const conflictsDir = join(dataDir, "conflict-test", "conflicts");
    expect(await fileExists(conflictsDir)).toBe(true);
    const entries = await readdir(conflictsDir);
    expect(entries.length).toBe(2);

    // Each entry should have blob and clock.json
    for (const entry of entries) {
      expect(await fileExists(join(conflictsDir, entry, "blob"))).toBe(true);
      expect(await fileExists(join(conflictsDir, entry, "clock.json"))).toBe(
        true,
      );
    }
  });

  test("multiple concurrent pushes before resolution accumulate entries in conflicts/", async () => {
    // Three devices A, B, C all start from the same base clock
    // Device A pushes first (becomes canonical)
    await push("accum-test", new Uint8Array([0xaa]), { base: 1, a: 1 });

    // Device B pushes with a clock concurrent to A (divergence -> 2 conflict entries)
    await push("accum-test", new Uint8Array([0xbb]), { base: 1, b: 1 });

    const conflictsDir = join(dataDir, "accum-test", "conflicts");
    let entries = await readdir(conflictsDir);
    expect(entries.length).toBe(2);

    // Device C's push is concurrent with existing conflict clocks -> returns 409
    const res = await push("accum-test", new Uint8Array([0xcc]), {
      base: 1,
      c: 1,
    });
    expect(res.status).toBe(409);

    // Still 2 entries (no accumulation after conflict)
    entries = await readdir(conflictsDir);
    expect(entries.length).toBe(2);
  });

  test("GET returns 409 in conflicted state with JSON body", async () => {
    await push("get-conflict", new Uint8Array([0xaa]), { a: 1 });
    await push("get-conflict", new Uint8Array([0xbb]), { b: 1 });

    const res = await fetch(`${baseUrl}/sync/get-conflict`);
    expect(res.status).toBe(409);

    const json = await res.json();
    expect(json).toHaveProperty("error");
  });

  test("POST into conflicted slot with clock descending from some but not all returns 409", async () => {
    await push("post-conflict", new Uint8Array([0xaa]), { a: 1 });
    await push("post-conflict", new Uint8Array([0xbb]), { b: 1 });

    // Push with clock that descends from {a:1} but not {b:1} — partial merge, should be rejected
    const res = await push("post-conflict", new Uint8Array([0xcc]), { a: 2 });
    expect(res.status).toBe(409);
  });

  test("POST with clock descending from all conflict clocks resolves the conflict", async () => {
    await push("resolve-test", new Uint8Array([0xaa]), { a: 1 });
    await push("resolve-test", new Uint8Array([0xbb]), { b: 1 });

    // Push with clock that descends from both conflict clocks
    const res = await push("resolve-test", new Uint8Array([0xff]), {
      a: 2,
      b: 2,
    });
    expect(res.status).toBe(200);

    // Conflict should be resolved - no conflicts dir entries
    const conflictsDir = join(dataDir, "resolve-test", "conflicts");
    if (await fileExists(conflictsDir)) {
      const entries = await readdir(conflictsDir);
      expect(entries.length).toBe(0);
    }

    // Canonical blob restored
    const getRes = await fetch(`${baseUrl}/sync/resolve-test`);
    expect(getRes.status).toBe(200);
    const returned = new Uint8Array(await getRes.arrayBuffer());
    expect(returned).toEqual(new Uint8Array([0xff]));
  });

  test("GET /sync/:sync_id/metadata returns conflicted: true when conflicts exist", async () => {
    await push("meta-conflict", new Uint8Array([0xaa]), { a: 1 });
    await push("meta-conflict", new Uint8Array([0xbb]), { b: 1 });

    const res = await fetch(`${baseUrl}/sync/meta-conflict/metadata`);
    expect(res.status).toBe(200);

    const json = await res.json();
    expect(json.conflicted).toBe(true);
  });

  test("GET /sync/:sync_id/metadata returns conflicted: false after resolution", async () => {
    await push("meta-resolve", new Uint8Array([0xaa]), { a: 1 });
    await push("meta-resolve", new Uint8Array([0xbb]), { b: 1 });

    // Resolve
    await push("meta-resolve", new Uint8Array([0xff]), { a: 2, b: 2 });

    const res = await fetch(`${baseUrl}/sync/meta-resolve/metadata`);
    expect(res.status).toBe(200);

    const json = await res.json();
    expect(json.conflicted).toBe(false);
  });

  test("after resolution, clock.json reflects the resolving clock and meta.json has correct blob_size and recent last_modified", async () => {
    await push("clock-meta-resolve", new Uint8Array([0xaa]), { a: 1 });
    await push("clock-meta-resolve", new Uint8Array([0xbb]), { b: 1 });

    const resolvedBlob = new Uint8Array([0xdd, 0xee, 0xff]);
    const resolvingClock = { a: 2, b: 2 };
    const before = Date.now();

    const resolveRes = await push(
      "clock-meta-resolve",
      resolvedBlob,
      resolvingClock,
    );
    expect(resolveRes.status).toBe(200);

    const after = Date.now();

    const clockJson = JSON.parse(
      await readFile(
        join(dataDir, "clock-meta-resolve", "clock.json"),
        "utf-8",
      ),
    );
    expect(clockJson).toEqual(resolvingClock);

    const meta = JSON.parse(
      await readFile(join(dataDir, "clock-meta-resolve", "meta.json"), "utf-8"),
    );
    expect(meta.blob_size).toBe(3);
    // last_modified is stored as Date.now() (milliseconds since epoch)
    expect(meta.last_modified).toBeGreaterThanOrEqual(before);
    expect(meta.last_modified).toBeLessThanOrEqual(after);
  });

  test("a second fast-forward push after resolution succeeds normally", async () => {
    // Create conflict
    await push("ff-after-resolve", new Uint8Array([0xaa]), { a: 1 });
    await push("ff-after-resolve", new Uint8Array([0xbb]), { b: 1 });

    // Resolve the conflict
    const resolveRes = await push("ff-after-resolve", new Uint8Array([0xff]), {
      a: 2,
      b: 2,
    });
    expect(resolveRes.status).toBe(200);

    // Fast-forward push after resolution — slot should behave as if never conflicted
    const ffBlob = new Uint8Array([0x11, 0x22]);
    const res = await push("ff-after-resolve", ffBlob, { a: 3, b: 2 });
    expect(res.status).toBe(200);

    // No conflicts directory should exist
    const conflictsDir = join(dataDir, "ff-after-resolve", "conflicts");
    if (await fileExists(conflictsDir)) {
      const entries = await readdir(conflictsDir);
      expect(entries.length).toBe(0);
    }

    // Blob is the fast-forwarded one
    const getRes = await fetch(`${baseUrl}/sync/ff-after-resolve`);
    expect(getRes.status).toBe(200);
    const returned = new Uint8Array(await getRes.arrayBuffer());
    expect(returned).toEqual(ffBlob);
  });
});
