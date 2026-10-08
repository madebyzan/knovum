import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { createServer } from "vite";

import { prepareCoreMedia } from "../runtime/media.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
await prepareCoreMedia(repository);
const server = await createServer({ root: repository });
await server.listen();
server.printUrls();
server.bindCLIShortcuts({ print: true });
for (const signal of ["SIGINT", "SIGTERM"]) {
  process.once(signal, async () => {
    await server.close();
    process.exit(0);
  });
}
