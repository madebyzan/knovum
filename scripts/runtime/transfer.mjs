import { createWriteStream, openSync, rmSync } from "node:fs";
import { Readable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { setTimeout } from "node:timers/promises";

import { sha256File } from "./files.mjs";

export async function downloadArtifact(
  source,
  file,
  headers = { "User-Agent": "Knovum runtime provisioner" },
) {
  if (!source.url.startsWith("https://")) throw new Error("Runtime downloads require HTTPS");
  for (let attempt = 1; ; attempt++) {
    try {
      // Retries must wait for the preceding attempt to finish and close its file.
      // oxlint-disable-next-line no-await-in-loop
      return await downloadOnce(source, file, headers);
    } catch (error) {
      const url = new URL(source.url);
      const detail = describeFailure(error);
      if (!isTransient(error) || attempt === 3)
        throw new Error(`Runtime download failed: ${url.origin}${url.pathname}: ${detail}`, { cause: error });
      console.warn(`Retrying runtime download (${attempt}/2): ${url.origin}${url.pathname}: ${detail}`);
      // oxlint-disable-next-line no-await-in-loop
      await setTimeout(attempt * 1000);
    }
  }
}

async function downloadOnce(source, file, headers) {
  const response = await fetchResponse(source.url, headers);
  let bytes = 0;
  const limit = source.size ?? 4 * 1024 ** 3;
  const bounded = new Transform({
    transform(chunk, _encoding, done) {
      bytes += chunk.length;
      done(bytes > limit ? new Error("Runtime download exceeds its size limit") : null, chunk);
    },
  });
  let descriptor;
  try {
    descriptor = openSync(file, "wx");
  } catch (error) {
    await response.body.cancel();
    throw error;
  }
  try {
    await pipeline(Readable.fromWeb(response.body), bounded, createWriteStream(file, { fd: descriptor }));
    if (source.size !== undefined && bytes !== source.size) throw new Error("Runtime download size differs");
    if (sha256File(file) !== source.sha256) throw new Error("Runtime download digest differs");
    return file;
  } catch (error) {
    rmSync(file, { force: true });
    throw error;
  }
}

async function fetchResponse(url, headers) {
  const response = await fetch(url, { headers, signal: AbortSignal.timeout(120_000) });
  if (!response.ok || !response.body) {
    await response.body?.cancel();
    const error = new Error(`HTTP ${response.status}`);
    if ([408, 429].includes(response.status) || response.status >= 500) error.code = "RUNTIME_HTTP_TRANSIENT";
    throw error;
  }
  if (!response.url.startsWith("https://")) {
    await response.body.cancel();
    throw new Error("Runtime redirect must retain HTTPS");
  }
  return response;
}

function underlyingError(error) {
  return error.cause?.errors?.[0] ?? error.cause ?? error;
}

function describeFailure(error) {
  const cause = underlyingError(error);
  return `${cause.code ? `${cause.code}: ` : ""}${cause.message || error.message}`;
}

function isTransient(error) {
  return (
    (error instanceof TypeError && error.message === "fetch failed") ||
    ["TimeoutError", "AbortError"].includes(error.name) ||
    /^(ECONNRESET|ECONNREFUSED|ETIMEDOUT|EAI_AGAIN|UND_ERR_.+|RUNTIME_HTTP_TRANSIENT)$/u.test(
      underlyingError(error).code ?? "",
    )
  );
}
