import { Hono } from "hono";
import {
  access,
  mkdir,
  readdir,
  readFile,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import { compareClocks, type VectorClock } from "./vector-clock.ts";

const MAX_BODY_SIZE = 50 * 1024 * 1024; // 50 MB
const VALID_SYNC_ID = /^[a-zA-Z0-9_-]+$/;

async function fileExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

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

async function atomicWrite(
  filePath: string,
  data: string | Uint8Array,
): Promise<void> {
  const tmpPath = filePath + ".tmp";
  await writeFile(tmpPath, data);
  await rename(tmpPath, filePath);
}

async function isConflicted(slotDir: string): Promise<boolean> {
  const conflictsDir = join(slotDir, "conflicts");
  try {
    const entries = await readdir(conflictsDir);
    return entries.length > 0;
  } catch {
    return false;
  }
}

async function getConflictClocks(slotDir: string): Promise<VectorClock[]> {
  const conflictsDir = join(slotDir, "conflicts");
  let entries: string[];
  try {
    entries = await readdir(conflictsDir);
  } catch {
    return [];
  }
  const clocks: VectorClock[] = [];
  for (const entry of entries) {
    const clockPath = join(conflictsDir, entry, "clock.json");
    try {
      const raw = await readFile(clockPath, "utf-8");
      clocks.push(JSON.parse(raw) as VectorClock);
    } catch {
      // skip entries with missing or invalid clock.json
    }
  }
  return clocks;
}

function descendsFromAll(
  incoming: VectorClock,
  clocks: VectorClock[],
): boolean {
  // If there are no conflict clocks but we're in conflicted state, don't resolve
  if (clocks.length === 0) return false;
  for (const clock of clocks) {
    const cmp = compareClocks(incoming, clock);
    if (cmp !== "a_descends_from_b" && cmp !== "identical") {
      return false;
    }
  }
  return true;
}

export function createApp(dataDir: string) {
  const app = new Hono();

  app.get("/sync/:sync_id", async (c) => {
    const syncId = c.req.param("sync_id");

    if (!VALID_SYNC_ID.test(syncId)) {
      return c.text("Invalid sync_id", 400);
    }

    const slotDir = join(dataDir, syncId);
    const blobPath = join(slotDir, "blob");

    // Check for conflicted state
    if (await isConflicted(slotDir)) {
      return c.json(
        {
          error: "conflict",
          message:
            "This sync slot is in a conflicted state. Use the metadata endpoint for details.",
        },
        409,
      );
    }

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

  app.get("/sync/:sync_id/metadata", async (c) => {
    const syncId = c.req.param("sync_id");

    if (!VALID_SYNC_ID.test(syncId)) {
      return c.text("Invalid sync_id", 400);
    }

    const slotDir = join(dataDir, syncId);
    const clockPath = join(slotDir, "clock.json");
    const metaPath = join(slotDir, "meta.json");

    try {
      const [clockRaw, metaRaw] = await Promise.all([
        readFile(clockPath, "utf-8"),
        readFile(metaPath, "utf-8"),
      ]);

      let clock: unknown;
      let meta: { blob_size: number; last_modified: number };
      try {
        clock = JSON.parse(clockRaw);
        meta = JSON.parse(metaRaw) as {
          blob_size: number;
          last_modified: number;
        };
      } catch {
        return c.text("Corrupt metadata", 500);
      }

      const conflicted = await isConflicted(slotDir);

      return c.json({
        vector_clock: clock,
        blob_size: meta.blob_size,
        last_modified: meta.last_modified,
        conflicted,
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
    const conflictsDir = join(slotDir, "conflicts");

    // Check Content-Length before reading the body
    const contentLength = Number(c.req.header("Content-Length") ?? "0");
    if (contentLength > MAX_BODY_SIZE) {
      return c.text("Payload too large", 413);
    }

    // Read raw body
    const body = new Uint8Array(await c.req.arrayBuffer());

    if (body.byteLength > MAX_BODY_SIZE) {
      return c.text("Payload too large", 413);
    }

    // Parse and validate vector clock from header
    const clockHeader = c.req.header("X-Vector-Clock");
    let incomingClock: VectorClock = {};
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
      incomingClock = parsed;
    }

    // Check if slot is already conflicted
    if (await isConflicted(slotDir)) {
      const conflictClocks = await getConflictClocks(slotDir);

      if (descendsFromAll(incomingClock, conflictClocks)) {
        // Resolve the conflict: clear conflicts dir, write canonical blob
        await rm(conflictsDir, { recursive: true, force: true });

        await atomicWrite(blobPath, body);
        await atomicWrite(clockPath, JSON.stringify(incomingClock));

        const meta = {
          last_modified: Date.now(),
          blob_size: body.byteLength,
        };
        await atomicWrite(metaPath, JSON.stringify(meta));

        return c.text("OK", 200);
      }

      // Incoming clock doesn't descend from all conflict clocks — reject
      return c.json(
        {
          error: "conflict",
          message:
            "Cannot push to a conflicted slot without a clock that descends from all conflict entries.",
        },
        409,
      );
    }

    // Non-conflicted slot: check for divergence
    let serverClockExists = false;
    let serverClock: VectorClock = {};
    try {
      const raw = await readFile(clockPath, "utf-8");
      serverClock = JSON.parse(raw) as VectorClock;
      serverClockExists = true;
    } catch {
      // No existing clock - first push
    }

    if (serverClockExists) {
      const comparison = compareClocks(incomingClock, serverClock);

      if (comparison === "concurrent") {
        // Divergence detected: move canonical blob to conflicts, store incoming
        await mkdir(conflictsDir, { recursive: true });

        // Move existing canonical blob to conflicts
        const existingId = crypto.randomUUID();
        const existingDir = join(conflictsDir, existingId);
        await mkdir(existingDir, { recursive: true });
        if (await fileExists(blobPath)) {
          await rename(blobPath, join(existingDir, "blob"));
        }
        await writeFile(
          join(existingDir, "clock.json"),
          JSON.stringify(serverClock),
        );

        // Store incoming blob as a new conflict entry
        const incomingId = crypto.randomUUID();
        const incomingDir = join(conflictsDir, incomingId);
        await mkdir(incomingDir, { recursive: true });
        await writeFile(join(incomingDir, "blob"), body);
        await writeFile(
          join(incomingDir, "clock.json"),
          JSON.stringify(incomingClock),
        );

        // Update top-level clock and meta (keep meta for metadata endpoint)
        await atomicWrite(clockPath, JSON.stringify(incomingClock));
        const meta = {
          last_modified: Date.now(),
          blob_size: body.byteLength,
        };
        await atomicWrite(metaPath, JSON.stringify(meta));

        return c.text("OK", 200);
      }
    }

    // Fast-forward or first push: write canonical blob
    // Ensure slot directory exists (after all validation to avoid orphaned dirs)
    await mkdir(slotDir, { recursive: true });

    await atomicWrite(blobPath, body);

    // Atomic write: clock.json
    await atomicWrite(clockPath, JSON.stringify(incomingClock));

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
