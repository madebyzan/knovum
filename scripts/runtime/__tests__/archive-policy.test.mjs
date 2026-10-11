import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { preflightArchive } from "../archive-policy.mjs";

const fixture = (name) => fileURLToPath(new URL(`./fixtures/archive-${name}.tar.gz`, import.meta.url));

test("valid runtime metadata above the child-process default buffer remains bounded and readable", () => {
  assert.throws(() => execFileSync("tar", ["-tvf", fixture("large")], { maxBuffer: 1024 ** 2 }), {
    code: "ENOBUFS",
  });
  assert.doesNotThrow(() => preflightArchive(fixture("large")));
});

for (const kind of ["unsafe", "link", "duplicate"])
  test(`runtime archive preflight rejects ${kind} entries before extraction`, () => {
    assert.throws(() => preflightArchive(fixture(kind)));
  });
