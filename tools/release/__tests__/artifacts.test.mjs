import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, rmSync, truncateSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  artifactSha256,
  MAX_ARTIFACT_BYTES,
  safeArtifactFilename,
  verifyReleaseArtifacts,
} from "../artifacts.mjs";

const directory = mkdtempSync(join(tmpdir(), "knovum-release-artifacts-test-"));

test.after(() => rmSync(directory, { recursive: true, force: true }));

test("hashes exact release bytes without loading the whole file", async () => {
  const file = join(directory, "application.tar.gz");
  const data = Buffer.alloc(100_000, 42);
  writeFileSync(file, data);
  assert.equal(await artifactSha256(file), createHash("sha256").update(data).digest("hex"));
});

test("empty and oversized release files fail before publication", async () => {
  const file = join(directory, "oversized.exe");
  writeFileSync(file, "");
  await assert.rejects(artifactSha256(file), /between 1 byte and 1 GiB/);
  truncateSync(file, MAX_ARTIFACT_BYTES + 1);
  await assert.rejects(artifactSha256(file), /between 1 byte and 1 GiB/);
});

test("release filenames cannot escape staging directories or inject options", () => {
  for (const name of ["../app.exe", "a/b.exe", "-o", "", "a\\b.exe", "..", "https://other/app"]) {
    assert.throws(() => safeArtifactFilename(name), /safe, flat filenames/);
  }
  assert.equal(
    safeArtifactFilename("Knovum_1.2.3_windows-x86_64-setup.exe"),
    "Knovum_1.2.3_windows-x86_64-setup.exe",
  );
});

test("hash mismatch is rejected before invoking a signature verifier", async () => {
  writeFileSync(join(directory, "bad.exe"), "changed");
  await assert.rejects(
    verifyReleaseArtifacts(directory, [{ file: "bad.exe", sha256: "wrong" }], ""),
    /hash mismatch/,
  );
});
