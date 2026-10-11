import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { verifyNativeRuntime } from "../native.mjs";

test("native manifest is bound to supplier inventory, not its own claims", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-native-"));
  try {
    const bytes = Buffer.from("verified native library");
    const file = {
      path: "onnxruntime.dll",
      size: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
    const source = { files: { "windows-x64": [file] } };
    const manifest = { schemaVersion: 1, target: "windows-x64", onnxRuntime: file.path, files: [file] };
    writeFileSync(join(root, file.path), bytes);
    assert.doesNotThrow(() => verifyNativeRuntime(root, manifest, source, "windows-x64"));
    assert.throws(() => verifyNativeRuntime(root, { ...manifest, files: [] }, source, "windows-x64"));
    assert.throws(() =>
      verifyNativeRuntime(root, { ...manifest, target: "macos-x64" }, source, "windows-x64"),
    );
    assert.throws(() =>
      verifyNativeRuntime(root, { ...manifest, onnxRuntime: "../outside.dll" }, source, "windows-x64"),
    );
    writeFileSync(join(root, file.path), Buffer.from("modified native library"));
    const forged = { ...file, sha256: createHash("sha256").update("modified native library").digest("hex") };
    assert.throws(() => verifyNativeRuntime(root, { ...manifest, files: [forged] }, source, "windows-x64"));
    rmSync(join(root, file.path));
    mkdirSync(join(root, file.path));
    assert.throws(() => verifyNativeRuntime(root, manifest, source, "windows-x64"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
