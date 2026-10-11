import assert from "node:assert/strict";
import test from "node:test";

import { generatedContractOutputs } from "../../contracts.mjs";
import { runtimeProfiles } from "../../runtime/profiles.mjs";
import { auditedPythonProjects } from "../python-audit.mjs";

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
