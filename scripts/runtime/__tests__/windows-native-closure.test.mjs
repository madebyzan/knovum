import assert from "node:assert/strict";
import { test } from "node:test";

import { validateWindowsImports } from "../windows-native-closure.mjs";
import { isMicrosoftRuntime, verifyMicrosoftRuntime } from "../windows-runtime-signature.mjs";

const inventory = [
  "bin/worker.exe",
  "bin/vcruntime140.dll",
  "compiled-libraries/audio.dll",
  "compiled-libraries/ggml.dll",
  "compiled-libraries/whisper/ggml.dll",
];
const system = new Set(["kernel32.dll"]);

test("only MSVC runtime families receive the Microsoft publisher signature policy", () => {
  for (const name of ["vcruntime140.dll", "VCRUNTIME140_1.dll", "msvcp140_atomic_wait.dll", "concrt140.dll"])
    assert(isMicrosoftRuntime(name));
  for (const name of ["onnxruntime.dll", "kernel32.dll", "evil-vcruntime140.dll", "vcruntime140.dll.exe"]) {
    assert(!isMicrosoftRuntime(name));
    assert.throws(() => verifyMicrosoftRuntime(name), /Unexpected Microsoft runtime filename/u);
  }
});

test("Windows native imports resolve only beside the importing DLL, in bin, or in the OS allowlist", () => {
  assert.doesNotThrow(() =>
    validateWindowsImports(
      "compiled-libraries/audio.dll",
      ["GGML.dll", "VCRUNTIME140.dll", "KERNEL32.dll", "api-ms-win-crt-runtime-l1-1-0.dll"],
      inventory,
      system,
    ),
  );
  assert.doesNotThrow(() =>
    validateWindowsImports("compiled-libraries/whisper/whisper.dll", ["ggml.dll"], inventory, system),
  );
  assert.throws(
    () => validateWindowsImports("bin/worker.exe", ["audio.dll"], inventory, system),
    /Unsealed Windows dependency/u,
  );
  assert.throws(
    () => validateWindowsImports("compiled-libraries/audio.dll", ["msvcp140.dll"], inventory, system),
    /Unsealed Windows dependency/u,
  );
  assert.throws(
    () => validateWindowsImports("bin/worker.exe", ["evil.dll"], inventory, system),
    /Unsealed Windows dependency/u,
  );
});

test("Windows imports reject missing inspection output and malformed DLL names", () => {
  for (const dependencies of [[], ["../kernel32.dll"], ["C:\\evil.dll"], ["api-ms-fake/evil.dll"]]) {
    assert.throws(() => validateWindowsImports("bin/worker.exe", dependencies, inventory, system));
  }
});
