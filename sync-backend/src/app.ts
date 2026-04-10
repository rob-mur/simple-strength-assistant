import { Hono } from "hono";
import { randomUUID } from "node:crypto";
import { readFile } from "node:fs/promises";
import { mkdir, rename, writeFile } from "node:fs/promises";
import { join } from "node:path";

const MAX_BODY_SIZE = 50 * 1024 * 1024; // 50 MB
const VALID_SYNC_ID = /^[a-zA-Z0-9_-]+$/;

function isVectorClock(v: unknown): v is Record<string, number> {
  return (
    typeof v === "object" &&
    v !== null &&
    !Array.isArray(v) &&
    Object.values(v as Record<string, unknown>).every(
      (x) => typeof x === "number",
    )
  );
}

async function atomicWrite(filePath: string, data: string | Uint8Array) {
  const tmpPath = `${filePath}.${randomUUID()}.tmp`;
  await writeFile(tmpPath, data);
  await rename(tmpPath, filePath);
}

export function createApp(dataDir: string) {
  const app = new Hono();

  app.get("/sync/:sync_id", async (c) => {
    const syncId = c.req.param("sync_id");

    if (!VALID_SYNC_ID.test(syncId)) {
      return c.text("Invalid sync_id", 400);
    }

    const blobPath = join(dataDir, syncId, "blob");

    try {
      const blob = await readFile(blobPath);
      return new Response(blob, {
        status: 200,
        headers: { "Content-Type": "application/octet-stream" },
      });
    } catch (err: unknown) {
      if (err instanceof Error && "code" in err && err.code === "ENOENT") {
        return c.text("Not found", 404);
      }
      throw err;
    }
  });

  app.post("/sync/:sync_id", async (c) => {
    const syncId = c.req.param("sync_id");

    if (!VALID_SYNC_ID.test(syncId)) {
      return c.text("Invalid sync_id", 400);
    }

    const slotDir = join(dataDir, syncId);
    const blobPath = join(slotDir, "blob");
    const clockPath = join(slotDir, "clock.json");
    const metaPath = join(slotDir, "meta.json");

    // Check Content-Length before reading the body
    const contentLength = Number(c.req.header("Content-Length") ?? "0");
    if (contentLength > MAX_BODY_SIZE) {
      return c.text("Payload too large", 413);
    }

    // Ensure slot directory exists
    await mkdir(slotDir, { recursive: true });

    // Read raw body
    const body = new Uint8Array(await c.req.arrayBuffer());

    if (body.byteLength > MAX_BODY_SIZE) {
      return c.text("Payload too large", 413);
    }

    // Parse vector clock from header
    const clockHeader = c.req.header("X-Vector-Clock");
    let clock: Record<string, number> = {};
    if (clockHeader) {
      let parsed: unknown;
      try {
        parsed = JSON.parse(clockHeader);
      } catch {
        return c.text("Invalid X-Vector-Clock header", 400);
      }
      if (!isVectorClock(parsed)) {
        return c.text("Invalid X-Vector-Clock header", 400);
      }
      clock = parsed;
    }

    // Atomic write: blob
    await atomicWrite(blobPath, body);

    // Atomic write: clock.json
    await atomicWrite(clockPath, JSON.stringify(clock));

    // Atomic write: meta.json
    const meta = {
      last_modified: Date.now(),
      blob_size: body.byteLength,
    };
    await atomicWrite(metaPath, JSON.stringify(meta));

    return c.text("OK", 200);
  });

  return app;
}
