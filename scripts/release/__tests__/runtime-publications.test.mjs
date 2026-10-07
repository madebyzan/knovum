import assert from "node:assert/strict";
import test from "node:test";

import { runtimeProfiles } from "../../runtime/profiles.mjs";
import { validateRuntimePublications } from "../runtime-publications.mjs";

function publication() {
  return {
    workerDigest: "a".repeat(64),
    pythonRuntimes: runtimeProfiles.map((profile) => ({
      schemaVersion: 1,
      profile,
      target: "macos-x64",
      version: "3.11.17",
      dependencyRequirementsSha256: "b".repeat(64),
      dependencyLockSha256: "c".repeat(64),
      sources: { global: "https://example.test/runtime", china: "https://china.example.test/runtime" },
      size: 123,
      sha256: "d".repeat(64),
      manifestSha256: "e".repeat(64),
    })),
  };
}

test("release binds all independent runtime publications to the target and worker", () => {
  validateRuntimePublications(publication(), "macos-x64");
  for (const mutate of [
    (value) => value.pythonRuntimes.pop(),
    (value) => (value.pythonRuntimes[1] = value.pythonRuntimes[0]),
    (value) => (value.pythonRuntimes[0].target = "macos-arm64"),
    (value) => (value.workerDigest = "invalid"),
  ]) {
    const value = publication();
    mutate(value);
    assert.throws(() => validateRuntimePublications(value, "macos-x64"));
  }
  assert.throws(() =>
    validateRuntimePublications({ localInference: publication().pythonRuntimes[0] }, "macos-x64"),
  );
});
