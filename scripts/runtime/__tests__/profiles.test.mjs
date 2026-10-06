import assert from "node:assert/strict";
import test from "node:test";

import { requirementsArguments, runtimeSources, runtimeTargetFromTriple } from "../profiles.mjs";

test("Assistant requirements select their independent project and exclude inference extras", () => {
  const args = requirementsArguments("/repository", "assistant");
  assert.equal(args[args.indexOf("--project") + 1], "/repository/python/runtimes/assistant");
  assert.equal(args.includes("--extra"), false);
  assert.throws(() => requirementsArguments("/repository", "unknown"), /Unsupported/);
});

test("Controlled sources bind the declared profile and platform", () => {
  const environment = Object.fromEntries(
    ["", "_DESCRIPTOR", "_EVIDENCE"].flatMap((suffix) => [
      [`KNOVUM_PYTHON_ASSISTANT${suffix}_WINDOWS_X64_URL`, `https://release.example/assistant${suffix}`],
      [`KNOVUM_PYTHON_ASSISTANT${suffix}_WINDOWS_X64_SHA256`, "a".repeat(64)],
    ]),
  );
  const sources = runtimeSources("windows-x64", "assistant", environment);
  assert.deepEqual(
    sources.map(({ kind }) => kind),
    ["PYTHON_RUNTIME", "PYTHON_RUNTIME_DESCRIPTOR", "PYTHON_RUNTIME_EVIDENCE"],
  );
  assert.throws(() => runtimeSources("windows-x64", "inference", environment), /Missing controlled FFMPEG/);
  assert.throws(() => runtimeSources("macos-arm64", "assistant", environment), /Missing controlled/);
});

test("Only the supported native release triples can select a runtime", () => {
  assert.equal(runtimeTargetFromTriple("aarch64-apple-darwin"), "macos-arm64");
  assert.equal(runtimeTargetFromTriple("x86_64-pc-windows-msvc"), "windows-x64");
  assert.equal(runtimeTargetFromTriple(undefined), undefined);
  assert.throws(() => runtimeTargetFromTriple("x86_64-apple-darwin"), /Unsupported/);
});
