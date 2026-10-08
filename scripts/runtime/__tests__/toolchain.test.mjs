import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";

import { sha256File } from "../files.mjs";
import { ensureUv } from "../toolchain.mjs";

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum toolchain 中文 "));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const target = "macos-x64";
  const archiveSha = "a".repeat(64);
  const directory = join(root, ".runtime-development", "tools", target, archiveSha);
  mkdirSync(directory, { recursive: true });
  const executable = join(directory, "uv");
  writeFileSync(executable, "official test binary");
  const source = {
    url: "https://example.invalid/uv.tar.gz",
    sha256: archiveSha,
    size: 1,
    entry: "uv/uv",
    executableSha256: sha256File(executable),
  };
  return { root, target, executable, source };
}

test("reuses only the exact pinned executable without network access", async (context) => {
  const sample = fixture(context);
  assert.equal(await ensureUv(sample.root, sample.target, sample.source), sample.executable);
});

test("a modified cached tool is rejected instead of executing or silently replacing it", async (context) => {
  const sample = fixture(context);
  writeFileSync(sample.executable, "modified binary");
  await assert.rejects(ensureUv(sample.root, sample.target, sample.source), /executable digest/);
});

test("tool installation rejects untrusted transport before downloading", async (context) => {
  const sample = fixture(context);
  rmSync(dirname(sample.executable), { recursive: true });
  sample.source.url = "http://example.invalid/uv.tar.gz";
  await assert.rejects(ensureUv(sample.root, sample.target, sample.source), /HTTPS/);
});
