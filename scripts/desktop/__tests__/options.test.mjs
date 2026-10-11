import { strict as assert } from "node:assert";
import { test } from "node:test";

import { buildOptions as parseOptions } from "../options.mjs";
const buildOptions = (args, command) => parseOptions(args, command, "x86_64-apple-darwin");

test("Cargo profile follows Tauri debug and custom profile options", () => {
  assert.equal(buildOptions(["--debug"], "build").profile, "dev");
  assert.equal(buildOptions(["--profile", "debugging"], "build").profile, "debugging");
  assert.equal(buildOptions([], "build").profile, "release");
  assert.equal(buildOptions([], "dev").profile, "dev");
});
test("production is explicit and cannot silently use a debug profile", () => {
  const options = buildOptions(["--release", "--no-bundle"], "build");
  assert.equal(options.production, true);
  assert.equal(options.bundle, false);
  assert.deepEqual(options.args, ["--no-bundle"]);
  assert.throws(() => buildOptions(["--release", "--debug"], "build"), /release profile/);
  assert.throws(() => buildOptions(["--profile"], "build"), /value/);
  assert.throws(() => buildOptions(["--release=true"], "build"), /does not accept/);
});

test("vendor target and debug aliases keep the Cargo and Tauri profiles aligned", () => {
  const options = buildOptions(["-d", "--target=x86_64-apple-darwin"], "build", "x86_64-apple-darwin");
  assert.equal(options.profile, "dev");
  assert.equal(options.explicitTarget, true);
  assert.throws(
    () => buildOptions(["-t", "aarch64-apple-darwin"], "build", "x86_64-apple-darwin"),
    /native target host/,
  );
});
