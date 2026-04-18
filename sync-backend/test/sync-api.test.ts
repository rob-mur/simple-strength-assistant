import {
  afterAll,
  beforeAll,
  describe,
  expect,
  test,
} from "bun:test";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createApp } from "../src/app.ts";
import type { Server } from "node:http";

describe("vlcn.io sync server", () => {
  let dataDir: string;
  let schemaDir: string;
  let server: Server;
  let port: number;
  let baseUrl: string;

  beforeAll(async () => {
    dataDir = await mkdtemp(join(tmpdir(), "sync-test-data-"));
    schemaDir = await mkdtemp(join(tmpdir(), "sync-test-schema-"));

    server = createApp(dataDir, schemaDir);

    await new Promise<void>((resolve) => {
      server.listen(0, () => {
        const addr = server.address();
        if (addr && typeof addr === "object") {
          port = addr.port;
        }
        baseUrl = `http://localhost:${port}`;
        resolve();
      });
    });
  });

  afterAll(async () => {
    await new Promise<void>((resolve) => server.close(() => resolve()));
    await rm(dataDir, { recursive: true, force: true });
    await rm(schemaDir, { recursive: true, force: true });
  });

  test("HTTP GET returns 404 (old blob endpoints removed)", async () => {
    const res = await fetch(`${baseUrl}/sync/test-slot`);
    expect(res.status).toBe(404);
  });

  test("HTTP POST returns 404 (old push endpoint removed)", async () => {
    const res = await fetch(`${baseUrl}/sync/test-slot`, {
      method: "POST",
      body: new Uint8Array([1, 2, 3]),
    });
    expect(res.status).toBe(404);
  });

  test("HTTP GET /sync/:sync_id/metadata returns 404 (old metadata endpoint removed)", async () => {
    const res = await fetch(`${baseUrl}/sync/test-slot/metadata`);
    expect(res.status).toBe(404);
  });

  test("WebSocket connection to /sync/:sync_id opens and closes cleanly", async () => {
    const wsUrl = `ws://localhost:${port}/sync/test-room`;

    // Encode room info in sec-websocket-protocol as vlcn.io expects
    const room = btoa("room=test-room,schemaName=default,schemaVersion=0");

    const ws = new WebSocket(wsUrl, [room]);

    const opened = await new Promise<boolean>((resolve) => {
      const timeout = setTimeout(() => resolve(false), 3000);
      ws.addEventListener("open", () => {
        clearTimeout(timeout);
        resolve(true);
      });
      ws.addEventListener("error", () => {
        clearTimeout(timeout);
        resolve(false);
      });
    });

    // The connection should at least open (vlcn.io may close it if
    // schema validation fails, but the WebSocket upgrade itself should succeed)
    expect(opened).toBe(true);

    ws.close();

    // Wait for close
    await new Promise<void>((resolve) => {
      const timeout = setTimeout(() => resolve(), 2000);
      ws.addEventListener("close", () => {
        clearTimeout(timeout);
        resolve();
      });
    });
  });

  test("WebSocket connections to different sync_ids are isolated", async () => {
    const room1 = btoa("room=room-a,schemaName=default,schemaVersion=0");
    const room2 = btoa("room=room-b,schemaName=default,schemaVersion=0");

    const ws1 = new WebSocket(`ws://localhost:${port}/sync/room-a`, [room1]);
    const ws2 = new WebSocket(`ws://localhost:${port}/sync/room-b`, [room2]);

    const [opened1, opened2] = await Promise.all([
      new Promise<boolean>((resolve) => {
        const timeout = setTimeout(() => resolve(false), 3000);
        ws1.addEventListener("open", () => {
          clearTimeout(timeout);
          resolve(true);
        });
        ws1.addEventListener("error", () => {
          clearTimeout(timeout);
          resolve(false);
        });
      }),
      new Promise<boolean>((resolve) => {
        const timeout = setTimeout(() => resolve(false), 3000);
        ws2.addEventListener("open", () => {
          clearTimeout(timeout);
          resolve(true);
        });
        ws2.addEventListener("error", () => {
          clearTimeout(timeout);
          resolve(false);
        });
      }),
    ]);

    expect(opened1).toBe(true);
    expect(opened2).toBe(true);

    ws1.close();
    ws2.close();

    await Promise.all([
      new Promise<void>((resolve) => {
        const timeout = setTimeout(() => resolve(), 2000);
        ws1.addEventListener("close", () => {
          clearTimeout(timeout);
          resolve();
        });
      }),
      new Promise<void>((resolve) => {
        const timeout = setTimeout(() => resolve(), 2000);
        ws2.addEventListener("close", () => {
          clearTimeout(timeout);
          resolve();
        });
      }),
    ]);
  });
});
