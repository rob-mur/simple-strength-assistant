import { createApp } from "./app.ts";

const dataDir = process.env["DATA_DIR"] ?? "/data";
const schemaDir = process.env["SCHEMA_DIR"] ?? "/schemas";
const port = Number(process.env["PORT"] ?? "3000");

const server = createApp(dataDir, schemaDir);

server.listen(port, () => {
  console.log(
    `vlcn.io sync server listening on port ${port}, DATA_DIR=${dataDir}`,
  );
});
