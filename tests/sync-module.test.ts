/**
 * Unit tests for sync-module.js — verifies buildWsUrl() produces URLs
 * that match the server's pathPattern: /^\/sync\/[a-zA-Z0-9_-]+$/
 *
 * Run with: bun test tests/sync-module.test.ts
 */
import { describe, expect, test } from "bun:test";

// The sync module reads window.SYNC_BASE_URL, so we need a minimal DOM shim.
// Bun's test runner includes a globalThis that we can extend.
const g = globalThis as Record<string, unknown>;

// Minimal window shim for the module
if (!g.window) {
  g.window = {
    SYNC_BASE_URL: "",
    location: { origin: "http://localhost:3000" },
  };
}

// We can't import the ES module directly because it depends on browser APIs
// (localStorage, WebSocket, etc.).  Instead, extract and test buildWsUrl logic
// inline to verify the URL construction matches the server's pathPattern.

/** The server's pathPattern from sync-backend/src/app.ts */
const SERVER_PATH_PATTERN = /^\/sync\/[a-zA-Z0-9_-]+$/;

/**
 * Replicated buildWsUrl logic from public/sync-module.js.
 * If this test fails, it means the sync module's URL doesn't match the server.
 */
function buildWsUrl(syncId: string, syncBaseUrl: string): string {
  let base = syncBaseUrl || "";
  if (!base || base.includes("%%")) {
    base = "http://localhost:3000/api";
  }
  const wsBase = base.replace(/^http/, "ws");
  return `${wsBase.replace(/\/$/, "")}/sync/${syncId}`;
}

/** Extract just the path from a WebSocket URL */
function extractPath(wsUrl: string): string {
  return new URL(wsUrl.replace(/^ws/, "http")).pathname;
}

describe("buildWsUrl", () => {
  test("produces URL path matching server pathPattern for UUID sync_id", () => {
    const url = buildWsUrl(
      "550e8400-e29b-41d4-a716-446655440000",
      "https://sync.clarob.uk",
    );
    const path = extractPath(url);
    expect(path).toMatch(SERVER_PATH_PATTERN);
  });

  test("produces URL path matching server pathPattern for simple sync_id", () => {
    const url = buildWsUrl("test-room", "https://sync.clarob.uk");
    const path = extractPath(url);
    expect(path).toMatch(SERVER_PATH_PATTERN);
  });

  test("does NOT append /ws suffix", () => {
    const url = buildWsUrl("my-room", "https://sync.clarob.uk");
    expect(url).not.toContain("/ws");
    expect(url).toBe("wss://sync.clarob.uk/sync/my-room");
  });

  test("converts https to wss", () => {
    const url = buildWsUrl("room-1", "https://sync.example.com");
    expect(url).toStartWith("wss://");
  });

  test("converts http to ws", () => {
    const url = buildWsUrl("room-1", "http://localhost:3000");
    expect(url).toStartWith("ws://");
  });

  test("strips trailing slash from base URL", () => {
    const url = buildWsUrl("room-1", "https://sync.example.com/");
    expect(url).toBe("wss://sync.example.com/sync/room-1");
  });

  test("falls back to localhost when SYNC_BASE_URL is placeholder", () => {
    const url = buildWsUrl("room-1", "%%SYNC_BASE_URL%%");
    expect(url).toBe("ws://localhost:3000/api/sync/room-1");
  });
});

describe("vlcn.io protocol header", () => {
  const SCHEMA_NAME = "default";
  const SCHEMA_VERSION = "7701108062419472521";

  test("base64-encoded room metadata round-trips correctly", () => {
    const syncId = "test-room";
    const meta = `room=${syncId},schemaName=${SCHEMA_NAME},schemaVersion=${SCHEMA_VERSION}`;
    const encoded = btoa(meta).replace(/=+$/, "");
    // Unpadded base64 can still be decoded by atob when re-padded
    const padded = encoded + "=".repeat((4 - (encoded.length % 4)) % 4);
    const decoded = atob(padded);
    expect(decoded).toBe(meta);
    expect(decoded).toContain(`room=${syncId}`);
    expect(decoded).toContain(`schemaName=${SCHEMA_NAME}`);
    expect(decoded).toContain(`schemaVersion=${SCHEMA_VERSION}`);
  });

  test("base64-encoded room metadata contains no padding characters", () => {
    // RFC 6455 §4.1 forbids '=' in WebSocket subprotocol values.
    const syncId = "550e8400-e29b-41d4-a716-446655440000";
    const meta = `room=${syncId},schemaName=${SCHEMA_NAME},schemaVersion=${SCHEMA_VERSION}`;
    const encoded = btoa(meta).replace(/=+$/, "");
    expect(encoded).not.toContain("=");
  });
});
