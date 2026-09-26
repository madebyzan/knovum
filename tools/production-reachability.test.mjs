import assert from "node:assert/strict";
import { test } from "node:test";

import { unreachableProductionModules } from "./check-production-reachability.mjs";

const module = (source, dependencies = []) => ({
  source,
  dependencies: dependencies.map((resolved) => ({ resolved })),
});

test("test-only imports do not make abandoned product code reachable", () => {
  const modules = [
    module("src/main.ts", ["src/App.vue"]),
    module("src/App.vue"),
    module("src/old-export.ts"),
    module("src/__tests__/export.test.ts", ["src/old-export.ts"]),
    module("src/contracts/generated/wire.ts"),
    module("src/vite-env.d.ts"),
  ];
  assert.deepEqual(unreachableProductionModules(modules), ["src/old-export.ts"]);
});

test("Worker URL imports participate in production reachability", () => {
  const modules = [module("src/main.ts"), module("src/text-worker.ts")];
  assert.deepEqual(
    unreachableProductionModules(modules, new Map([["src/main.ts", ["src/text-worker.ts"]]])),
    [],
  );
});

test("missing product entry is an analysis failure", () => {
  assert.throws(() => unreachableProductionModules([]), /missing src\/main\.ts/u);
});
