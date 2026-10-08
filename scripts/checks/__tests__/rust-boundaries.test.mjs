import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { inspectRustBoundaries } from "../rust-boundaries.mjs";

function fixture(t, files) {
  const root = mkdtempSync(join(tmpdir(), "knovum-rust-boundaries-"));
  t.after(() => rmSync(root, { recursive: true }));
  for (const [name, source] of Object.entries(files)) {
    const path = join(root, name);
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, source);
  }
  return root;
}

test("local resources cannot import AI policy", (t) => {
  const root = fixture(t, {
    "src-tauri/src/local_resources/catalog.rs": "use crate::ai::contracts::Model;\n",
  });
  assert.deepEqual(inspectRustBoundaries(root), [
    "src-tauri/src/local_resources/catalog.rs imports the AI capability",
  ]);
});

test("durable Runs cannot import concrete capabilities", (t) => {
  const root = fixture(t, {
    "src-tauri/src/spaces/runs/runtime.rs":
      "crate::spaces::speech::execute();\nuse crate::spaces::{runs, workflows};\n",
  });
  assert.deepEqual(inspectRustBoundaries(root), [
    "src-tauri/src/spaces/runs/runtime.rs imports concrete capability crate::spaces::speech",
    "src-tauri/src/spaces/runs/runtime.rs imports concrete capability crate::spaces::workflows",
  ]);
});

test("composition hooks keep neutral owners independent", (t) => {
  const root = fixture(t, {
    "src-tauri/src/local_resources/catalog.rs": 'pub const SPEC_ID: &str = "model-v1";\n',
    "src-tauri/src/spaces/runs/runtime.rs": "hooks.execute_step(app, pool, claim).await;\n",
    "src-tauri/src/spaces/runs/tests/fixture.rs": "crate::spaces::speech::fixture();\n",
  });
  assert.deepEqual(inspectRustBoundaries(root), []);
});

test("authorization and captured workflows cannot regain catalog or Run routing", (t) => {
  const root = fixture(t, {
    "src-tauri/src/spaces/authorization/grant.rs": "crate::spaces::actions::definitions();\n",
    "src-tauri/src/spaces/workflows/execution/mod.rs":
      "crate::spaces::action_execution::execute_durable();\n",
  });
  assert.deepEqual(inspectRustBoundaries(root), [
    "src-tauri/src/spaces/authorization/grant.rs imports the Action catalog",
    "src-tauri/src/spaces/workflows/execution/mod.rs dispatches a non-captured Run capability",
  ]);
});
