import {
  createServer,
  type IncomingMessage,
  type ServerResponse,
  type Server,
} from "node:http";
import { attachWebsocketServer, type Config } from "@vlcn.io/ws-server";

function setCorsHeaders(res: ServerResponse): void {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "*");
}

/**
 * Create and configure the vlcn.io sync server.
 *
 * The server accepts WebSocket connections at `/sync/<sync_id>`.  The
 * `sync_id` path segment is used directly as the vlcn.io room identifier,
 * providing per-room isolation with changesets persisted to `dataDir`.
 */
export function createApp(dataDir: string, schemaDir: string): Server {
  const server = createServer((req: IncomingMessage, res: ServerResponse) => {
    setCorsHeaders(res);

    // Handle CORS preflight requests.
    if (req.method === "OPTIONS") {
      res.writeHead(204);
      res.end();
      return;
    }

    // Health check for load balancer probes.
    if (req.url === "/health") {
      res.writeHead(200);
      res.end("ok");
      return;
    }

    // No HTTP endpoints — only WebSocket upgrades are served.
    res.writeHead(404);
    res.end("Not found");
  });

  const config: Config = {
    dbFolder: dataDir,
    schemaFolder: schemaDir,
    pathPattern: /^\/sync\/[a-zA-Z0-9_-]+$/,
  };

  attachWebsocketServer(server, config);

  server.on("error", (err) => {
    console.error("Server error:", err);
    process.exit(1);
  });

  return server;
}
