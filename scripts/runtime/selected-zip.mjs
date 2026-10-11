import { on } from "node:events";
import { createWriteStream, mkdirSync } from "node:fs";
import { dirname } from "node:path";
import { pipeline } from "node:stream/promises";
import { promisify } from "node:util";

import yauzl from "yauzl";

// A lazy archive cursor advances only after the selected file is durably streamed.
export async function extractSelectedZip(archive, selected) {
  const zip = await promisify(yauzl.open)(archive, { lazyEntries: true, validateEntrySizes: true });
  const found = new Set();
  try {
    const entries = on(zip, "entry", { close: ["end"] });
    zip.readEntry();
    for await (const [entry] of entries) {
      const destination = selected.get(entry.fileName);
      if (destination) {
        const type = (entry.externalFileAttributes >>> 16) & 0o170000;
        if (
          found.has(entry.fileName) ||
          ![0, 0o100000].includes(type) ||
          entry.uncompressedSize < 1 ||
          entry.uncompressedSize > 256 * 1024 ** 2
        )
          throw new Error("Selected supplier entry must be a unique bounded regular file");
        found.add(entry.fileName);
        mkdirSync(dirname(destination), { recursive: true });
        const stream = await promisify(zip.openReadStream.bind(zip))(entry);
        await pipeline(stream, createWriteStream(destination, { flags: "wx" }));
      }
      zip.readEntry();
    }
    if (found.size !== selected.size) throw new Error("Supplier archive lacks selected entries");
  } finally {
    zip.close();
  }
}
