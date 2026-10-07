import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { readRuntimeDescriptor, validateRuntimeDescriptor } from "../distribution.mjs";
import { validatePackagedResources, workerDigest } from "../packaged-resources.mjs";
import { runtimeProfiles } from "../profiles.mjs";

function fixture(context) {
  const repository = mkdtempSync(join(tmpdir(), "knovum-small-package-"));
  context.after(() => rmSync(repository, { recursive: true, force: true }));
  const source = join(repository, "python/src/knovum_inference");
  const resources = join(repository, "app/resources");
  const worker = join(resources, "python-worker/knovum_inference");
  const publication = join(repository, "runtime-artifacts/resources/windows-x64");
  for (const path of [source, worker, publication]) mkdirSync(path, { recursive: true });
  writeFileSync(join(source, "server.py"), "application Worker\n");
  copyFileSync(join(source, "server.py"), join(worker, "server.py"));
  const descriptor = {
    schemaVersion: 1,
    profile: "resources",
    target: "windows-x64",
    version: "3.11.17",
    dependencyRequirementsSha256: "a".repeat(64),
    dependencyLockSha256: "b".repeat(64),
    archiveUrl: "https://example.test/immutable-v1/python-runtime-windows-x64.tar.gz",
    size: 123,
    sha256: "c".repeat(64),
    manifestSha256: "d".repeat(64),
  };
  const path = join(resources, "python-resources-descriptor.json");
  const save = () => writeFileSync(path, JSON.stringify(descriptor));
  save();
  copyFileSync(path, join(publication, "runtime-descriptor.json"));
  for (const profile of runtimeProfiles.filter((name) => name !== "resources")) {
    const profilePublication = join(repository, "runtime-artifacts", profile, "windows-x64");
    mkdirSync(profilePublication, { recursive: true });
    const encoded = JSON.stringify({ ...descriptor, profile });
    writeFileSync(join(resources, `python-${profile}-descriptor.json`), encoded);
    writeFileSync(join(profilePublication, "runtime-descriptor.json"), encoded);
  }
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

test("runtime descriptor rejects unsupported schema versions", (context) => {
  const sample = fixture(context);
  for (const schemaVersion of [0, 2, 3]) {
    sample.descriptor.schemaVersion = schemaVersion;
    sample.save();
    assert.throws(
      () => readRuntimeDescriptor(sample.path, "windows-x64", "resources"),
      /Invalid Python runtime descriptor/,
    );
  }
});

test("desktop payload verifies every application-owned Python capability", (context) => {
  const sample = fixture(context);
  const source = join(sample.repository, "python/src/knovum_document_processing");
  const packaged = join(sample.resources, "python-worker/knovum_document_processing");
  mkdirSync(source);
  writeFileSync(join(source, "__init__.py"), "document capability\n");
  assert.throws(sample.verify, /Worker differs/);
  mkdirSync(packaged);
  copyFileSync(join(source, "__init__.py"), join(packaged, "__init__.py"));
  const before = sample.verify().workerDigest;
  writeFileSync(join(source, "__init__.py"), "updated documents\n");
  assert.throws(sample.verify, /Worker differs/);
  copyFileSync(join(source, "__init__.py"), join(packaged, "__init__.py"));
  assert.notEqual(sample.verify().workerDigest, before);
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
    () => readRuntimeDescriptor(sample.path, "windows-arm64", "resources"),
    /Invalid Python runtime descriptor/,
  );
  sample.descriptor.archiveUrl = "http://example.test/runtime.tar.gz";
  sample.save();
  assert.throws(
    () => readRuntimeDescriptor(sample.path, "windows-x64", "resources"),
    /Runtime archive URL must be HTTPS/,
  );
});

test("Worker changes have their own identity without changing the runtime descriptor", (context) => {
  const sample = fixture(context);
  const before = workerDigest(sample.worker);
  writeFileSync(join(sample.worker, "server.py"), "updated Worker\n");
  assert.notEqual(workerDigest(sample.worker), before);
  assert.equal(
    readRuntimeDescriptor(sample.path, "windows-x64", "resources").sha256,
    sample.descriptor.sha256,
  );
});

test("an Agent Runtime descriptor cannot replace the resources descriptor", (context) => {
  const sample = fixture(context);
  sample.descriptor.profile = "agent";
  sample.save();
  assert.throws(sample.verify, /Invalid Python runtime descriptor/);
});

test("publication owns one archive URL and rejects regional descriptors", (context) => {
  const sample = fixture(context);
  const { archiveUrl: previousUrl, ...identity } = sample.descriptor;
  const sources = { global: previousUrl, china: "https://china.example.test/runtime.tar.gz" };
  const descriptor = { ...identity, archiveUrl: "https://downloads.example.test/runtime.tar.gz" };
  validateRuntimeDescriptor(descriptor, "windows-x64", "resources");
  assert.throws(() => validateRuntimeDescriptor({ ...descriptor, sources }, "windows-x64", "resources"));
  assert.throws(() => validateRuntimeDescriptor({ ...identity, sources }, "windows-x64", "resources"));
  for (const archiveUrl of [
    "http://example.test/runtime",
    "https://user:secret@example.test/runtime",
    "https://example.test/runtime#fragment",
  ]) {
    assert.throws(() => validateRuntimeDescriptor({ ...descriptor, archiveUrl }, "windows-x64", "resources"));
  }
});
