import assert from "node:assert/strict";
import test from "node:test";

import {
  requirementsArguments,
  runtimeProfiles,
  runtimeSources,
  runtimeTargetFromTriple,
} from "../profiles.mjs";

test("production profiles export only their own independent frozen dependency project", () => {
  assert.deepEqual(runtimeProfiles, [
    "resources",
    "light-inference",
    "torch-inference",
    "voice-generation",
    "agent",
  ]);
  for (const profile of runtimeProfiles) {
    const args = requirementsArguments("/repository", profile);
    assert.equal(args[args.indexOf("--project") + 1], `/repository/python/runtimes/${profile}`);
    assert.equal(args.includes("--extra"), false);
    assert.equal(args.includes("--no-emit-package"), profile === "voice-generation");
  }
  assert.throws(() => requirementsArguments("/repository", "inference"), /Unsupported/);
});

test("Agent Runtime requirements select their independent project and exclude inference extras", () => {
  const args = requirementsArguments("/repository", "agent");
  assert.equal(args[args.indexOf("--project") + 1], "/repository/python/runtimes/agent");
  assert.equal(args.includes("--extra"), false);
  assert.throws(() => requirementsArguments("/repository", "unknown"), /Unsupported/);
});

test("Controlled sources bind the declared profile and platform", () => {
  const environment = Object.fromEntries(
    ["", "_DESCRIPTOR", "_EVIDENCE"].flatMap((suffix) => [
      [`KNOVUM_PYTHON_AGENT${suffix}_WINDOWS_X64_URL`, `https://release.example/agent${suffix}`],
      [`KNOVUM_PYTHON_AGENT${suffix}_WINDOWS_X64_SHA256`, "a".repeat(64)],
    ]),
  );
  const sources = runtimeSources("windows-x64", "agent", environment);
  assert.deepEqual(
    sources.map(({ kind }) => kind),
    ["PYTHON_RUNTIME", "PYTHON_RUNTIME_DESCRIPTOR", "PYTHON_RUNTIME_EVIDENCE"],
  );
  assert.throws(
    () => runtimeSources("windows-x64", "light-inference", environment),
    /Missing controlled FFMPEG/,
  );
  assert.throws(() => runtimeSources("macos-arm64", "agent", environment), /Missing controlled/);
});

test("Only the supported native release triples can select a runtime", () => {
  assert.equal(runtimeTargetFromTriple("aarch64-apple-darwin"), "macos-arm64");
  assert.equal(runtimeTargetFromTriple("x86_64-pc-windows-msvc"), "windows-x64");
  assert.equal(runtimeTargetFromTriple(undefined), undefined);
  assert.equal(runtimeTargetFromTriple("x86_64-apple-darwin"), "macos-x64");
  assert.throws(() => runtimeTargetFromTriple("aarch64-pc-windows-msvc"), /Unsupported/);
});
