import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createApp } from "../src/app.ts";

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
    const res = await fetch(`${baseUrl}/sync/../../etc`, {
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
    const res = await fetch(`${baseUrl}/sync/too-large`, {
      method: "POST",
      body: new Uint8Array([1]),
      headers: {
        "Content-Type": "application/octet-stream",
        "Content-Length": String(100 * 1024 * 1024),
        "X-Vector-Clock": JSON.stringify({ n: 1 }),
      },
    });
    expect(res.status).toBe(413);
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
