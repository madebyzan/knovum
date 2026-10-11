import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { sha256File } from "../files.mjs";
import { validateNativePackage } from "../native-package.mjs";

function fixture(context) {
  const directory = mkdtempSync(join(tmpdir(), "native-package-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "bin"));
  const files = ["bin/knovum-inference-worker.exe", "onnxruntime.dll"].map((path) => {
    const bytes = Buffer.from(path);
    writeFileSync(join(directory, path), bytes);
    return { path, size: bytes.length, sha256: sha256File(join(directory, path)) };
  });
  const manifest = { schemaVersion: 1, target: "windows-x64", onnxRuntime: "onnxruntime.dll", files };
  writeFileSync(join(directory, "native-runtime-manifest.json"), JSON.stringify(manifest));
  return { directory, manifest };
}
test("sealed native package matches an externally trusted build inventory", (context) => {
  const { directory, manifest } = fixture(context);
  assert.deepEqual(validateNativePackage(directory, "windows-x64", manifest), manifest);
  assert.throws(() => validateNativePackage(directory, "macos-x64", manifest));
  assert.throws(() => validateNativePackage(directory, "windows-x64", { ...manifest, files: [] }));
});
for (const change of ["bytes", "extra", "link", "missing"])
  test(`native package rejects ${change}`, (context) => {
    const { directory, manifest } = fixture(context);
    if (change === "bytes") writeFileSync(join(directory, "onnxruntime.dll"), "changed");
    if (change === "extra") writeFileSync(join(directory, "unsealed"), "extra");
    if (change === "link") symlinkSync(join(directory, "onnxruntime.dll"), join(directory, "alias"));
    if (change === "missing") rmSync(join(directory, "bin/knovum-inference-worker.exe"));
    assert.throws(() => validateNativePackage(directory, "windows-x64", manifest));
  });
