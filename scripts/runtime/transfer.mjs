import { createWriteStream } from "node:fs";
import { Readable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";

import { sha256File } from "./files.mjs";

export async function downloadArtifact(
  source,
  file,
  headers = { "User-Agent": "Knovum runtime provisioner" },
) {
  if (!source.url.startsWith("https://")) throw new Error("Runtime downloads require HTTPS");
  const response = await fetch(source.url, { headers });
  if (!response.ok || !response.body) throw new Error(`Runtime download failed: HTTP ${response.status}`);
  if (!response.url.startsWith("https://")) throw new Error("Runtime redirect must retain HTTPS");
  let bytes = 0;
  const limit = source.size ?? 4 * 1024 ** 3;
  const bounded = new Transform({
    transform(chunk, _encoding, done) {
      bytes += chunk.length;
      done(bytes > limit ? new Error("Runtime download exceeds its size limit") : null, chunk);
    },
  });
  await pipeline(Readable.fromWeb(response.body), bounded, createWriteStream(file, { flags: "wx" }));
  if (source.size !== undefined && bytes !== source.size) throw new Error("Runtime download size differs");
  if (sha256File(file) !== source.sha256) throw new Error("Runtime download digest differs");
  return file;
}
