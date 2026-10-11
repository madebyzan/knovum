import { on } from "node:events";
import { createWriteStream, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { pipeline } from "node:stream/promises";
import { promisify } from "node:util";

import yauzl from "yauzl";

export async function extractSourceZip(archive, destination, { prefix, select = () => true }) {
  const zip = await promisify(yauzl.open)(archive, { lazyEntries: true, validateEntrySizes: true });
  try {
    const entries = on(zip, "entry", { close: ["end"] });
    zip.readEntry();
    for await (const [entry] of entries) {
      const parts = entry.fileName.split("/");
      if (parts[0] !== prefix || parts.includes("..") || entry.fileName.includes("\\"))
        throw new Error("Unsafe inference source entry");
      const relative = parts.slice(1).join("/");
      if (!relative || !select(relative)) {
        zip.readEntry();
        continue;
      }
      const path = join(destination, entry.fileName);
      await extractEntry(zip, entry, path);
      zip.readEntry();
    }
  } finally {
    zip.close();
  }
  return join(destination, prefix);
}

async function extractEntry(zip, entry, path) {
  const type = (entry.externalFileAttributes >>> 16) & 0o170000;
  if (entry.fileName.endsWith("/")) {
    if (![0, 0o040000].includes(type)) throw new Error("Inference source directory is not regular");
    mkdirSync(path, { recursive: true });
    return;
  }
  if (![0, 0o100000].includes(type) || entry.uncompressedSize > 256 * 1024 ** 2)
    throw new Error("Inference build source requires bounded regular files");
  mkdirSync(dirname(path), { recursive: true });
  const stream = await promisify(zip.openReadStream.bind(zip))(entry);
  await pipeline(stream, createWriteStream(path, { flags: "wx" }));
}
