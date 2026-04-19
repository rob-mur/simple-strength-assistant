import { createServer, type Server } from "node:http";
import { attachWebsocketServer, type Config } from "@vlcn.io/ws-server";

/**
 * Create and configure the vlcn.io sync server.
 *
 * The server accepts WebSocket connections at `/sync/<sync_id>`.  The
 * `sync_id` path segment is used directly as the vlcn.io room identifier,
 * providing per-room isolation with changesets persisted to `dataDir`.
 */
export function createApp(dataDir: string, schemaDir: string): Server {
  const server = createServer((_req, res) => {
    // No HTTP endpoints — only WebSocket upgrades are served.
    res.writeHead(404);
    res.end("Not found");
  });

  const config: Config = {
    dbFolder: dataDir,
    schemaFolder: schemaDir,
    pathPattern: /\/sync\/[a-zA-Z0-9_-]+/,
  };

  attachWebsocketServer(server, config);

  return server;
}
