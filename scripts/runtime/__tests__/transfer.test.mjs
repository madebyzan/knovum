import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { downloadArtifact } from "../transfer.mjs";

const bytes = Buffer.from("pinned supplier archive");
const source = {
  url: "https://supplier.example/archive.tar.gz",
  size: bytes.length,
  sha256: createHash("sha256").update(bytes).digest("hex"),
};

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-transfer-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  return join(root, "archive");
}

function response(body = bytes, status = 200) {
  return { ok: status === 200, status, url: source.url, body: new Response(body).body };
}

test("a transient connection failure can recover without restarting preparation", async (context) => {
  const file = fixture(context);
  let calls = 0;
  context.mock.method(globalThis, "fetch", async () => {
    if (++calls === 1)
      throw new TypeError("fetch failed", {
        cause: Object.assign(new Error("connection reset"), { code: "ECONNRESET" }),
      });
    return response();
  });
  await downloadArtifact(source, file);
  assert.deepEqual(readFileSync(file), bytes);
  assert.equal(calls, 2);
});

test("exhausted downloads identify the supplier and underlying connection error", async (context) => {
  const file = fixture(context);
  context.mock.method(globalThis, "fetch", async () => {
    throw new TypeError("fetch failed", {
      cause: Object.assign(new Error("connect timed out"), { code: "ETIMEDOUT" }),
    });
  });
  await assert.rejects(
    downloadArtifact(source, file),
    (error) => error.message.includes(source.url) && error.message.includes("ETIMEDOUT"),
  );
  assert.equal(existsSync(file), false);
});

test("a rejected archive is removed and an integrity failure is not retried", async (context) => {
  const file = fixture(context);
  let calls = 0;
  context.mock.method(globalThis, "fetch", async () => {
    calls++;
    return response(Buffer.alloc(bytes.length));
  });
  await assert.rejects(downloadArtifact(source, file), /digest differs/u);
  assert.equal(existsSync(file), false);
  assert.equal(calls, 1);
});

test("downloads never remove an existing destination", async (context) => {
  const file = fixture(context);
  writeFileSync(file, "existing artifact");
  context.mock.method(globalThis, "fetch", async () => response());
  await assert.rejects(downloadArtifact(source, file));
  assert.equal(readFileSync(file, "utf8"), "existing artifact");
});
