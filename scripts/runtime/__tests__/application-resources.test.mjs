import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { validateApplicationResources, workerDigest } from "../application-resources.mjs";
import { readRuntimeDescriptor } from "../distribution.mjs";

function fixture(context) {
  const repository = mkdtempSync(join(tmpdir(), "knovum-small-package-"));
  context.after(() => rmSync(repository, { recursive: true, force: true }));
  const source = join(repository, "python/src/knovum_inference");
  const resources = join(repository, "app/resources");
  const worker = join(resources, "python-worker/knovum_inference");
  const publication = join(repository, "runtime-artifacts/inference/windows-x64");
  for (const path of [source, worker, publication]) mkdirSync(path, { recursive: true });
  writeFileSync(join(source, "server.py"), "application Worker\n");
  copyFileSync(join(source, "server.py"), join(worker, "server.py"));
  const descriptor = {
    schemaVersion: 2,
    profile: "inference",
    target: "windows-x64",
    version: "3.11.17",
    dependencyRequirementsSha256: "a".repeat(64),
    dependencyLockSha256: "b".repeat(64),
    sources: {
      global: "https://example.test/immutable-v1/python-runtime-windows-x64.tar.gz",
      china: "https://china.example.test/immutable-v1/python-runtime-windows-x64.tar.gz",
    },
    size: 123,
    sha256: "c".repeat(64),
    manifestSha256: "d".repeat(64),
  };
  const path = join(resources, "python-inference-descriptor.json");
  const save = () => writeFileSync(path, JSON.stringify(descriptor));
  save();
  copyFileSync(path, join(publication, "runtime-descriptor.json"));
  const agentPublication = join(repository, "runtime-artifacts/agent/windows-x64");
  mkdirSync(agentPublication, { recursive: true });
  const agentDescriptor = JSON.stringify({ ...descriptor, profile: "agent" });
  writeFileSync(join(resources, "python-agent-descriptor.json"), agentDescriptor);
  writeFileSync(join(agentPublication, "runtime-descriptor.json"), agentDescriptor);
  return {
    repository,
    resources,
    source,
    worker,
    path,
    descriptor,
    save,
    verify: () => validateApplicationResources(resources, repository, "windows-x64"),
  };
}

test("desktop payload contains only owned Worker and trusted runtime descriptor", (context) => {
  const sample = fixture(context);
  assert.equal(sample.verify().workerDigest, workerDigest(sample.source));
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
  assert.equal(sample.verify().workerDigest, before);
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
    () => readRuntimeDescriptor(sample.path, "windows-arm64", "inference"),
    /Invalid Python runtime descriptor/,
  );
  sample.descriptor.sources.china = "http://example.test/runtime.tar.gz";
  sample.save();
  assert.throws(
    () => readRuntimeDescriptor(sample.path, "windows-x64", "inference"),
    /Runtime archive sources must be HTTPS/,
  );
});

test("Worker changes have their own identity without changing the runtime descriptor", (context) => {
  const sample = fixture(context);
  const before = workerDigest(sample.worker);
  writeFileSync(join(sample.worker, "server.py"), "updated Worker\n");
  assert.notEqual(workerDigest(sample.worker), before);
  assert.equal(
    readRuntimeDescriptor(sample.path, "windows-x64", "inference").sha256,
    sample.descriptor.sha256,
  );
});

test("an Agent Runtime descriptor cannot replace the inference descriptor", (context) => {
  const sample = fixture(context);
  sample.descriptor.profile = "agent";
  sample.save();
  assert.throws(sample.verify, /Invalid Python runtime descriptor/);
});
