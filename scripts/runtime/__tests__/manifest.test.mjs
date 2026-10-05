import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { listRuntimeFiles } from "../files.mjs";
import { validatePythonRuntime } from "../manifest.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const commit = "a".repeat(40);

function write(base, path, bytes) {
  const file = join(base, path);
  mkdirSync(join(file, ".."), { recursive: true });
  writeFileSync(file, bytes);
}

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-validation-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const python = join(root, "python");
  const runtime = join(root, "runtime");
  const lock = `source = { git = "https://github.com/OpenBMB/VoxCPM.git?rev=${commit}#${commit}" }\n`;
  write(python, "uv.lock", lock);
  const target = "macos-arm64";
  const source = { url: "https://example.test/cpython.tar.gz", sha256: "b".repeat(64), size: 42 };
  write(
    python,
    "runtime-source.json",
    JSON.stringify({ targets: { [target]: source }, uvVersion: "0.12.23", standaloneRelease: "20261003" }),
  );
  const native = { sources: [{ fileName: "native.zip", name: "native", sha256: hash("native"), size: 6 }] };
  write(python, "compiled-source-evidence.json", JSON.stringify(native));
  write(python, "src/knovum_inference/__main__.py", "pass\n");
  write(runtime, "bin/python3", "interpreter");
  chmodSync(join(runtime, "bin/python3"), 0o755);
  write(runtime, "packages/dependency.py", "pass\n");
  write(runtime, "runtime-requirements.txt", "requirements");
  const evidence = join(root, "build-evidence");
  const producer = {
    schemaVersion: 1,
    target,
    version: "3.11.17",
    uvVersion: "0.12.23",
    standaloneRelease: "20261003",
    dependencyLockSha256: hash(lock),
    appCommit: commit,
    interpreterSource: source,
    relocationVerified: true,
    signing: { mode: "adhoc", identity: "-", compiledBinaryCount: 1 },
    voxcpm: {
      commit,
      tree: commit,
      wheelSha256: "c".repeat(64),
      sourceSha256: hash("source"),
      requirementsSha256: hash("requirements"),
      buildRequirementsSha256: hash("build"),
    },
  };
  write(evidence, "producer.json", JSON.stringify(producer));
  for (const [name, bytes] of [
    ["dependency-evidence.json", "{}"],
    ["sbom.cdx.json", "{}"],
    ["requirements.txt", "requirements"],
    ["build-requirements.txt", "build"],
    ["voxcpm-source.tar", "source"],
    ["compiled-source-evidence.json", JSON.stringify(native)],
    ["compiled-sources/native.zip", "native"],
  ])
    write(evidence, name, bytes);
  function manifest() {
    const files = listRuntimeFiles(runtime, { excludeManifest: true }).map((path) => {
      const bytes = readFileSync(join(runtime, path));
      return { path, size: bytes.length, sha256: hash(bytes) };
    });
    write(
      runtime,
      "runtime-manifest.json",
      JSON.stringify({
        schemaVersion: 2,
        runtimeId: "cpython",
        version: "3.11.17",
        dependencyRequirementsSha256: hash("requirements"),
        voxcpmCommit: commit,
        target,
        executable: "bin/python3",
        packageRoot: "packages",
        files,
      }),
    );
  }
  manifest();
  return {
    python,
    evidence,
    runtime,
    producer,
    manifest,
    write,
    validate: () => validatePythonRuntime(runtime, target, join(python, "uv.lock")),
  };
}

test("consumer binds interpreter, runtime requirements and external corresponding source bytes", (context) => {
  const sample = fixture(context);
  sample.validate();
});

test("application Worker changes do not invalidate the third-party runtime", (context) => {
  const sample = fixture(context);
  sample.write(sample.python, "src/knovum_inference/__main__.py", "different worker\n");
  sample.validate();
});

test("changing an interpreter input remains invalid after recomputing file hashes", (context) => {
  const sample = fixture(context);
  sample.producer.interpreterSource = { ...sample.producer.interpreterSource, size: 43 };
  sample.write(sample.evidence, "producer.json", JSON.stringify(sample.producer));
  sample.manifest();
  assert.throws(sample.validate, /producer identity differs/);
});

test("producer cannot claim a different frozen VoxCPM source", (context) => {
  const sample = fixture(context);
  sample.producer.voxcpm.commit = "d".repeat(40);
  sample.write(sample.evidence, "producer.json", JSON.stringify(sample.producer));
  sample.manifest();
  assert.throws(sample.validate, /VoxCPM build identity differs/);
});

test("requirements evidence must agree with producer digest", (context) => {
  const sample = fixture(context);
  sample.write(sample.evidence, "requirements.txt", "changed");
  sample.manifest();
  assert.throws(sample.validate, /producer digest differs/);
});

test("runtime changes are rejected before trusting producer metadata", (context) => {
  const sample = fixture(context);
  sample.write(sample.runtime, "bin/python3", "corrupt");
  assert.throws(sample.validate, /file does not match/);
});

test("runtime archive rejects application Worker files", (context) => {
  const sample = fixture(context);
  sample.write(sample.runtime, "packages/knovum_inference/__main__.py", "pass\n");
  sample.manifest();
  assert.throws(sample.validate, /exclude application Worker/);
});

test("unrelated development lock changes do not invalidate runtime identity", (context) => {
  const sample = fixture(context);
  sample.write(
    sample.python,
    "uv.lock",
    `# quality dependency changed\nsource = { git = "https://github.com/OpenBMB/VoxCPM.git?rev=${commit}#${commit}" }\n`,
  );
  sample.validate();
});
