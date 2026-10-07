import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

test("Spaces handlers and ACL manifest declare the same commands", () => {
  const manifest = readFileSync(new URL("../../../src-tauri/build.rs", import.meta.url), "utf8");
  const declaration = manifest.match(
    /"spaces",\s*tauri_build::InlinedPlugin::new\(\)\s*\.commands\(&\[([\s\S]*?)\]\)/,
  );
  assert.ok(declaration, "Spaces ACL command declaration must exist");
  const declared = [...declaration[1].matchAll(/"(\w+)"/g)].map((match) => match[1]);
  const registry = new URL("../../../src-tauri/src/spaces/command_registry/", import.meta.url);
  const registered = readdirSync(registry)
    .filter((name) => name.endsWith(".rs") && name !== "mod.rs")
    .flatMap((name) => {
      const source = readFileSync(new URL(name, registry), "utf8");
      const handler = source.match(/tauri::generate_handler!\[([\s\S]*?)\]/);
      assert.ok(handler, `${name} must declare its command handler`);
      return [...handler[1].matchAll(/::(\w+)\s*,/g)].map((match) => match[1]);
    });

  assert.deepEqual(declared.toSorted(), registered.toSorted());
});
