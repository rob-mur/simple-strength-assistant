import { createApp } from "./app.ts";

const dataDir = process.env["DATA_DIR"] ?? "/data";
const port = Number(process.env["PORT"] ?? "3000");

const app = createApp(dataDir);

const server = Bun.serve({
  fetch: app.fetch,
  port,
});

console.log(
  `Sync backend listening on port ${server.port}, DATA_DIR=${dataDir}`,
);
