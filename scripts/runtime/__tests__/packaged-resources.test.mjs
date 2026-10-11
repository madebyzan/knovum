import assert from "node:assert/strict";
import { copyFileSync, cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { readRuntimeDescriptor, validateRuntimeDescriptor } from "../distribution.mjs";
import { validateMediaLicenseReview } from "../media-license-policy.mjs";
import { mediaSource, writeMediaManifest } from "../media-manifest.mjs";
import { validatePackagedResources, workerDigest } from "../packaged-resources.mjs";

function fixture(context) {
  const repository = mkdtempSync(join(tmpdir(), "knovum-small-package-"));
  context.after(() => rmSync(repository, { recursive: true, force: true }));
  const source = join(repository, "python/src/knovum_agent_runtime");
  const resources = join(repository, "app/resources");
  const worker = join(resources, "python-worker/knovum_agent_runtime");
  const publication = join(repository, "runtime-artifacts/agent/windows-x64");
  const checkout = fileURLToPath(new URL("../../../", import.meta.url));
  for (const name of ["python", "scripts/runtime"]) mkdirSync(join(repository, name), { recursive: true });
  copyFileSync(join(checkout, "python/runtime-source.json"), join(repository, "python/runtime-source.json"));
  cpSync(join(checkout, "scripts/runtime"), join(repository, "scripts/runtime"), { recursive: true });
  const media = join(resources, "bundled-runtime/windows-x64");
  mkdirSync(join(media, "bin"), { recursive: true });
  mkdirSync(join(media, "build-evidence"));
  writeFileSync(join(media, "bin/ffmpeg.exe"), "controlled FFmpeg executable");
  copyFileSync(
    join(repository, "scripts/runtime/media-license-review.json"),
    join(media, "build-evidence/media-license-review.json"),
  );
  writeMediaManifest(repository, media, "windows-x64", {
    schemaVersion: 1,
    appCommit: "a".repeat(40),
    compiledBinaryFiles: ["bin/ffmpeg.exe"],
    target: "windows-x64",
    version: "9.0.2",
    relocationVerified: true,
    signing: { mode: "unsigned", compiledBinaryCount: 1 },
    source: mediaSource(repository, "windows-x64").source,
    licenseReview: validateMediaLicenseReview(repository, "windows-x64", {
      requireApproved: false,
    }),
  });
  for (const path of [source, worker, publication]) mkdirSync(path, { recursive: true });
  writeFileSync(join(source, "server.py"), "application Worker\n");
  copyFileSync(join(source, "server.py"), join(worker, "server.py"));
  const descriptor = {
    schemaVersion: 1,
    profile: "agent",
    target: "windows-x64",
    version: "3.11.17",
    dependencyRequirementsSha256: "a".repeat(64),
    dependencyLockSha256: "b".repeat(64),
    archiveUrl: "https://example.test/immutable-v1/python-runtime-windows-x64.tar.gz",
    size: 123,
    sha256: "c".repeat(64),
    manifestSha256: "d".repeat(64),
  };
  const path = join(resources, "python-agent-descriptor.json");
  const save = () => writeFileSync(path, JSON.stringify(descriptor));
  save();
  copyFileSync(path, join(publication, "runtime-descriptor.json"));
  return {
    repository,
    resources,
    source,
    worker,
    path,
    descriptor,
    save,
    verify: () => validatePackagedResources(resources, repository, "windows-x64"),
  };
}

test("desktop payload contains only owned Worker and trusted runtime descriptor", (context) => {
  const sample = fixture(context);
  assert.equal(sample.verify().workerDigest, workerDigest(join(sample.repository, "python/src")));
});

test("desktop payload verifies its bundled core media executable", (context) => {
  const sample = fixture(context);
  writeFileSync(join(sample.resources, "bundled-runtime/windows-x64/bin/ffmpeg.exe"), "altered FFmpeg");
  assert.throws(sample.verify, /Media runtime file differs/u);
});

test("runtime descriptor rejects unsupported schema versions", (context) => {
  const sample = fixture(context);
  for (const schemaVersion of [0, 2, 3]) {
    sample.descriptor.schemaVersion = schemaVersion;
    sample.save();
    assert.throws(
      () => readRuntimeDescriptor(sample.path, "windows-x64", "agent"),
      /Invalid Python runtime descriptor/,
    );
  }
});

test("package rejects Python business packages outside the retained Agent", (context) => {
  const sample = fixture(context);
  const unexpected = join(sample.resources, "python-worker/knovum_inference");
  mkdirSync(unexpected);
  writeFileSync(join(unexpected, "server.py"), "obsolete inference");
  assert.throws(sample.verify, /Worker differs|Agent/u);
});

test("package rejects altered Worker or descriptor bytes", (context) => {
  const sample = fixture(context);
  writeFileSync(join(sample.worker, "server.py"), "other Worker");
  assert.throws(sample.verify, /Worker differs/);
  copyFileSync(join(sample.source, "server.py"), join(sample.worker, "server.py"));
  sample.descriptor.sha256 = "e".repeat(64);
  sample.save();
  assert.throws(sample.verify, /descriptor differs/);
});

test("package rejects accidental third-party interpreter inclusion", (context) => {
  const sample = fixture(context);
  writeFileSync(join(sample.resources, "python.exe"), "large runtime");
  assert.throws(sample.verify, /exclude the third-party Python runtime/);
});

test("descriptor rejects unsupported platform and insecure download source", (context) => {
  const sample = fixture(context);
  assert.throws(
    () => readRuntimeDescriptor(sample.path, "windows-arm64", "agent"),
    /Invalid Python runtime descriptor/,
  );
  sample.descriptor.archiveUrl = "http://example.test/runtime.tar.gz";
  sample.save();
  assert.throws(
    () => readRuntimeDescriptor(sample.path, "windows-x64", "agent"),
    /Runtime archive URL must be HTTPS/,
  );
});

test("Worker changes have their own identity without changing the runtime descriptor", (context) => {
  const sample = fixture(context);
  const before = workerDigest(sample.worker);
  writeFileSync(join(sample.worker, "server.py"), "updated Worker\n");
  assert.notEqual(workerDigest(sample.worker), before);
  assert.equal(readRuntimeDescriptor(sample.path, "windows-x64", "agent").sha256, sample.descriptor.sha256);
});

test("obsolete inference descriptors cannot replace the Agent descriptor", (context) => {
  const sample = fixture(context);
  sample.descriptor.profile = "torch-inference";
  sample.save();
  assert.throws(sample.verify, /Invalid Python runtime descriptor/);
});

test("publication owns one archive URL and rejects regional descriptors", (context) => {
  const sample = fixture(context);
  const { archiveUrl: previousUrl, ...identity } = sample.descriptor;
  const sources = { global: previousUrl, china: "https://china.example.test/runtime.tar.gz" };
  const descriptor = { ...identity, archiveUrl: "https://downloads.example.test/runtime.tar.gz" };
  validateRuntimeDescriptor(descriptor, "windows-x64", "agent");
  assert.throws(() => validateRuntimeDescriptor({ ...descriptor, sources }, "windows-x64", "agent"));
  assert.throws(() => validateRuntimeDescriptor({ ...identity, sources }, "windows-x64", "agent"));
  for (const archiveUrl of [
    "http://example.test/runtime",
    "https://user:secret@example.test/runtime",
    "https://example.test/runtime#fragment",
  ]) {
    assert.throws(() => validateRuntimeDescriptor({ ...descriptor, archiveUrl }, "windows-x64", "agent"));
  }
});
