import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { prepareDevelopmentRuntime } from "../development.mjs";

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-development-runtime-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  return join(root, "a".repeat(64));
}

function validate(directory) {
  assert.equal(readFileSync(join(directory, "valid"), "utf8"), "yes");
}

test("an unchanged valid runtime is reused without invoking its producer", async (context) => {
  const output = fixture(context);
  mkdirSync(output);
  writeFileSync(join(output, "valid"), "yes");
  let produced = false;
  await prepareDevelopmentRuntime({
    output,
    validate,
    produce: () => {
      produced = true;
    },
  });
  assert.equal(produced, false);
});

test("invalid publications are rejected without deleting a runtime outside coordinated ownership", async (context) => {
  const output = fixture(context);
  mkdirSync(output);
  writeFileSync(join(output, "valid"), "no");
  await assert.rejects(
    prepareDevelopmentRuntime({ output, validate, produce: () => assert.fail("producer ran") }),
  );
  assert.equal(existsSync(output), true);
});

test("failed staging validation leaves no partial runtime", async (context) => {
  const output = fixture(context);
  const staging = join(output, "..", `.${"a".repeat(64)}.test.installing`);
  await assert.rejects(
    prepareDevelopmentRuntime({
      output,
      validate,
      stagingId: () => "test",
      produce: (directory) => {
        mkdirSync(directory);
        writeFileSync(join(directory, "valid"), "no");
      },
    }),
  );
  assert.equal(existsSync(output), false);
  assert.equal(existsSync(staging), false);
});

test("a concurrent validated publication wins and losing staging is removed", async (context) => {
  const output = fixture(context);
  const staging = join(output, "..", `.${"a".repeat(64)}.test.installing`);
  await prepareDevelopmentRuntime({
    output,
    validate,
    stagingId: () => "test",
    produce: (directory) => {
      mkdirSync(directory);
      writeFileSync(join(directory, "valid"), "yes");
      mkdirSync(output);
      writeFileSync(join(output, "valid"), "yes");
    },
  });
  validate(output);
  assert.equal(existsSync(staging), false);
});
