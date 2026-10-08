import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

import { commit, fixture } from "./manifest-fixture.mjs";

test("consumer binds interpreter, runtime requirements and frozen voice source bytes", (context) => {
  const sample = fixture(context);
  sample.validate();
});

test("runtime manifest and producer reject unsupported schema versions", (context) => {
  const sample = fixture(context);
  const path = join(sample.runtime, "runtime-manifest.json");
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  for (const schemaVersion of [0, 2, 3]) {
    writeFileSync(path, JSON.stringify({ ...manifest, schemaVersion }));
    assert.throws(sample.validate, /manifest is invalid/);
  }
  sample.manifest();
  for (const schemaVersion of [0, 2, 3]) {
    sample.write(sample.evidence, "producer.json", JSON.stringify({ ...sample.producer, schemaVersion }));
    assert.throws(sample.validate, /producer identity differs/);
  }
});

test("application Worker changes do not invalidate the third-party runtime", (context) => {
  const sample = fixture(context);
  sample.write(sample.python, "src/knovum_inference/__main__.py", "different worker\n");
  sample.validate();
});

test("changing an interpreter input remains invalid after recomputing file hashes", (context) => {
  const sample = fixture(context);
  sample.producer.nativeSupplier.packages[0].size = 43;
  sample.write(sample.evidence, "producer.json", JSON.stringify(sample.producer));
  sample.manifest();
  assert.throws(sample.validate, /producer identity differs/);
});

test("producer cannot claim a different frozen VoxCPM source", (context) => {
  const sample = fixture(context);
  sample.producer.voxcpm.version = "d".repeat(40);
  sample.write(sample.evidence, "producer.json", JSON.stringify(sample.producer));
  sample.manifest();
  assert.throws(sample.validate, /VoxCPM native source identity differs/);
});

test("requirements evidence must agree with producer digest", (context) => {
  const sample = fixture(context);
  sample.write(sample.evidence, "requirements.txt", "changed");
  sample.manifest();
  assert.throws(sample.validate, /requirements differ from the producer evidence/);
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

test("any changed dependency lock invalidates the frozen runtime identity", (context) => {
  const sample = fixture(context);
  sample.write(
    sample.python,
    "uv.lock",
    `# quality dependency changed\nsource = { git = "https://github.com/OpenBMB/VoxCPM.git?rev=${commit}#${commit}" }\n`,
  );
  assert.throws(sample.validate, /dependency lock differs/);
});

test("light runtime retains native corresponding-source verification independently of VoxCPM", (context) => {
  const sample = fixture(context, "light-inference");
  sample.validate();
  sample.write(sample.evidence, "compiled-sources/native.zip", "changed");
  assert.throws(sample.validate, /Native source archive differs/);
});

test("native installed inventory is bound to the producer record", (context) => {
  const sample = fixture(context);
  sample.write(sample.evidence, "native-packages.json", "{}");
  assert.throws(sample.validate, /Native package inventory differs/);
});
