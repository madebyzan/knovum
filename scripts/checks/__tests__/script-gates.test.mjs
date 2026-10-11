import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { generatedContractOutputs } from "../../contracts.mjs";
import { runtimeProfiles } from "../../runtime/profiles.mjs";
import { auditedPythonProjects } from "../python-audit.mjs";
import { scriptUnitTests } from "../scripts.mjs";

test("one script gate discovers new owned unit tests and excludes integration/fixtures", (t) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-script-gate-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  for (const path of [
    "runtime/__tests__/new.test.mjs",
    "desktop/__tests__/dev.test.mjs",
    "release/__tests__/signatures.integration.mjs",
    "release/__tests__/fixture.mjs",
  ]) {
    mkdirSync(join(root, path, ".."), { recursive: true });
    writeFileSync(join(root, path), "");
  }
  assert.deepEqual(scriptUnitTests(root), [
    join(root, "desktop/__tests__/dev.test.mjs"),
    join(root, "runtime/__tests__/new.test.mjs"),
  ]);
});

test("contract drift gate includes model constraints, Python models and all TS owners", () => {
  for (const path of [
    "python/schemas/agent_runtime.schema.json",
    "src-tauri/crates/inference-contracts/inference.schema.json",
    "python/src/knovum_agent_runtime/contracts",
    "src/features/dictation/contracts/generated",
    "src/features/spaces/contracts/generated",
    "src/shared/contracts/generated",
  ])
    assert.ok(generatedContractOutputs.includes(path), path);
  assert.equal(new Set(generatedContractOutputs).size, generatedContractOutputs.length);
});

test("PR dependency audits include the root and every independent runtime profile", () => {
  assert.deepEqual(auditedPythonProjects, ["python", ...runtimeProfiles.map((p) => `python/runtimes/${p}`)]);
});
