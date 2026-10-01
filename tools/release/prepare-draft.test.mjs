import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";

import { releaseTargets } from "./manifest.mjs";
import { prepareDraft } from "./prepare-draft.mjs";
import { releaseFixture } from "./test-fixtures.mjs";

test("only creates a draft after all signatures and repository visibility pass", (context) => {
  const calls = [];
  let manifestFile;
  const result = prepareDraft(releaseFixture(context), (command, args) => {
    calls.push({ command, args });
    if (command === "gh" && args[0] === "repo") return "false\n";
    if (command === "gh" && args[1] === "upload") {
      manifestFile = args.at(-1);
      assert.equal(JSON.parse(readFileSync(manifestFile, "utf8")).version, "0.0.2");
    }
    return "";
  });
  const verified = releaseTargets.length;
  assert.deepEqual(
    calls.slice(0, verified).map(({ command }) => command),
    releaseTargets.map(() => "minisign"),
  );
  assert.equal(calls[verified + 1].args[1], "create");
  assert.ok(calls[verified + 1].args.includes("--draft"));
  assert.equal(calls[verified + 2].args[1], "upload");
  assert.ok(calls.every(({ args }) => !args.includes("--clobber")));
  assert.equal(calls.length, verified + 3);
  assert.equal(result.tag, "v0.0.2");
  assert.equal(existsSync(manifestFile), false);
});

test("signature rejection results in zero GitHub requests", (context) => {
  const calls = [];
  assert.throws(
    () =>
      prepareDraft(releaseFixture(context), (command) => {
        calls.push(command);
        throw new Error("signature verification failed");
      }),
    /verification/,
  );
  assert.deepEqual(calls, ["minisign"]);
});

test("private distribution repository is rejected without writes", (context) => {
  const writes = [];
  assert.throws(
    () =>
      prepareDraft(releaseFixture(context), (command, args) => {
        if (command === "gh" && args[0] === "repo") return "true\n";
        if (command === "gh") writes.push(args);
        return "";
      }),
    /public/,
  );
  assert.deepEqual(writes, []);
});

test("a conflicting release fails instead of replacing existing artifacts", (context) => {
  let uploaded = false;
  assert.throws(
    () =>
      prepareDraft(releaseFixture(context), (command, args) => {
        if (command === "gh" && args[0] === "repo") return "false\n";
        if (command === "gh" && args[1] === "create") throw new Error("release already exists");
        if (command === "gh" && args[1] === "upload") uploaded = true;
        return "";
      }),
    /already exists/,
  );
  assert.equal(uploaded, false);
});

test("an upload error is not reported as success and does not publish the draft", (context) => {
  const calls = [];
  assert.throws(
    () =>
      prepareDraft(releaseFixture(context), (command, args) => {
        calls.push({ command, args });
        if (command === "gh" && args[0] === "repo") return "false\n";
        if (command === "gh" && args[1] === "upload") throw new Error("upload failed");
        return "";
      }),
    /upload failed/,
  );
  assert.ok(calls.every(({ args }) => args[1] !== "edit"));
});
