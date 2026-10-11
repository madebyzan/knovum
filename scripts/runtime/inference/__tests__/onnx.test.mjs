import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { verifyOnnxRuntime } from "../onnx.mjs";

test("inference manifest is bound to supplier inventory, not its own claims", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-inference-"));
  try {
    const bytes = Buffer.from("verified inference library");
    const file = {
      path: "onnxruntime.dll",
      size: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
    const source = { files: { "windows-x64": [file] } };
    const manifest = { schemaVersion: 1, target: "windows-x64", onnxRuntime: file.path, files: [file] };
    writeFileSync(join(root, file.path), bytes);
    assert.doesNotThrow(() => verifyOnnxRuntime(root, manifest, source, "windows-x64"));
    assert.throws(() => verifyOnnxRuntime(root, { ...manifest, files: [] }, source, "windows-x64"));
    assert.throws(() => verifyOnnxRuntime(root, { ...manifest, target: "macos-x64" }, source, "windows-x64"));
    assert.throws(() =>
      verifyOnnxRuntime(root, { ...manifest, onnxRuntime: "../outside.dll" }, source, "windows-x64"),
    );
    writeFileSync(join(root, file.path), Buffer.from("modified inference library"));
    const forged = {
      ...file,
      sha256: createHash("sha256").update("modified inference library").digest("hex"),
    };
    assert.throws(() => verifyOnnxRuntime(root, { ...manifest, files: [forged] }, source, "windows-x64"));
    rmSync(join(root, file.path));
    mkdirSync(join(root, file.path));
    assert.throws(() => verifyOnnxRuntime(root, manifest, source, "windows-x64"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
