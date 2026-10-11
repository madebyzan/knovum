import { strict as assert } from "node:assert";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { seal, verify } from "../resources.mjs";
test("a final resource cannot authorize its own corrupted bytes", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-bundle-"));
  try {
    mkdirSync(join(root, "bin"));
    writeFileSync(join(root, "bin/worker"), "original");
    const trusted = seal(root, "manifest.json", { target: "macos-x64" });
    verify(root, "manifest.json", trusted);
    writeFileSync(join(root, "bin/worker"), "modified");
    assert.throws(() => verify(root, "manifest.json", trusted), /bytes differ/);
    seal(root, "manifest.json", { target: "macos-x64" });
    assert.throws(() => verify(root, "manifest.json", trusted), /manifest differs/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
