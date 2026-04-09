import { Hono } from "hono";
import { existsSync } from "node:fs";
import { mkdir, rename, writeFile } from "node:fs/promises";
import { join } from "node:path";

export function createApp(dataDir: string) {
  const app = new Hono();

  app.get("/sync/:sync_id", async (c) => {
    const syncId = c.req.param("sync_id");
    const blobPath = join(dataDir, syncId, "blob");

    if (!existsSync(blobPath)) {
      return c.text("Not found", 404);
    }

    const blob = await Bun.file(blobPath).arrayBuffer();
    return new Response(blob, {
      status: 200,
      headers: { "Content-Type": "application/octet-stream" },
    });
  });

  app.post("/sync/:sync_id", async (c) => {
    const syncId = c.req.param("sync_id");
    const slotDir = join(dataDir, syncId);
    const blobPath = join(slotDir, "blob");
    const tmpPath = join(slotDir, "blob.tmp");
    const clockPath = join(slotDir, "clock.json");
    const metaPath = join(slotDir, "meta.json");

    // Ensure slot directory exists
    await mkdir(slotDir, { recursive: true });

    // Read raw body
    const body = new Uint8Array(await c.req.arrayBuffer());

    // Parse vector clock from header
    const clockHeader = c.req.header("X-Vector-Clock");
    const clock = clockHeader ? JSON.parse(clockHeader) : {};

    // Atomic write: write to temp then rename
    await writeFile(tmpPath, body);
    await rename(tmpPath, blobPath);

    // Write clock.json
    await writeFile(clockPath, JSON.stringify(clock));

    // Write meta.json
    const meta = {
      last_modified: Date.now(),
      blob_size: body.byteLength,
    };
    await writeFile(metaPath, JSON.stringify(meta));

    return c.text("OK", 200);
  });

  return app;
}
