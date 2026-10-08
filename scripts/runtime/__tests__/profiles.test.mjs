import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repository = fileURLToPath(new URL("../../../", import.meta.url)).replace(/\/$/u, "");

import {
  developmentDirectory,
  requirementsArguments,
  producerBytes,
  runtimeProfiles,
  runtimeTargetFromTriple,
} from "../profiles.mjs";

test("development identity binds the producer recipe as well as dependency locks", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-identity-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const name of ["python/runtimes/resources", "python/native-locks/base", "scripts/runtime"])
    mkdirSync(join(root, name), { recursive: true });
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  cpSync(join(repository, "scripts/runtime"), join(root, "scripts/runtime"), { recursive: true });
  writeFileSync(join(root, "python/runtimes/resources/uv.lock"), "frozen dependencies");
  writeFileSync(join(root, "python/native-locks/base/macos-x64.json"), "frozen native packages");
  writeFileSync(join(root, "scripts/runtime/native.py"), "first producer");
  const first = developmentDirectory(root, "macos-x64", "resources");
  assert.equal(developmentDirectory(root, "macos-x64", "resources"), first);
  writeFileSync(join(root, "scripts/runtime/native.py"), "corrected producer");
  assert.notEqual(developmentDirectory(root, "macos-x64", "resources"), first);
  writeFileSync(join(root, "scripts/runtime/native.py"), "first producer");
  writeFileSync(join(root, "scripts/runtime/provision.mjs"), "changed distribution consumer");
  writeFileSync(join(root, "scripts/runtime/unrelated.mjs"), "unrelated tool");
  assert.equal(developmentDirectory(root, "macos-x64", "resources"), first);
});

test("producer identity follows the selected resource's actual source closure", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-producer-scope-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "python"));
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  cpSync(join(repository, "scripts/runtime"), join(root, "scripts/runtime"), { recursive: true });
  const before = new Map(
    [...runtimeProfiles, "media"].map((profile) => [profile, producerBytes(root, profile)]),
  );
  writeFileSync(join(root, "scripts/runtime/voice_bridge.cpp"), "changed voice supplier");
  for (const profile of [...runtimeProfiles, "media"]) {
    assert.equal(
      producerBytes(root, profile).equals(before.get(profile)),
      profile !== "voice-generation",
      profile,
    );
  }
  writeFileSync(join(root, "scripts/runtime/media_files.py"), "changed FFmpeg closure exporter");
  assert.equal(producerBytes(root, "media").equals(before.get("media")), false);
  assert.equal(producerBytes(root, "resources").equals(before.get("resources")), true);
});

test("production profiles export only their own independent frozen dependency project", () => {
  assert.deepEqual(runtimeProfiles, [
    "resources",
    "light-inference",
    "torch-inference",
    "voice-generation",
    "agent",
  ]);
  for (const profile of runtimeProfiles) {
    const args = requirementsArguments(repository, profile);
    assert.equal(args[args.indexOf("--project") + 1], `${repository}/python/runtimes/${profile}`);
    assert.equal(args.includes("--extra"), false);
    assert.equal(args.includes("--no-emit-package"), true);
  }
  assert.throws(() => requirementsArguments(repository, "inference"), /Unsupported/);
});

test("Agent Runtime requirements select their independent project and exclude inference extras", () => {
  const args = requirementsArguments(repository, "agent");
  assert.equal(args[args.indexOf("--project") + 1], `${repository}/python/runtimes/agent`);
  assert.equal(args.includes("--extra"), false);
  assert.throws(() => requirementsArguments(repository, "unknown"), /Unsupported/);
});

test("Only the supported native release triples can select a runtime", () => {
  assert.equal(runtimeTargetFromTriple("aarch64-apple-darwin"), "macos-arm64");
  assert.equal(runtimeTargetFromTriple("x86_64-pc-windows-msvc"), "windows-x64");
  assert.equal(runtimeTargetFromTriple(undefined), undefined);
  assert.equal(runtimeTargetFromTriple("x86_64-apple-darwin"), "macos-x64");
  assert.throws(() => runtimeTargetFromTriple("aarch64-pc-windows-msvc"), /Unsupported/);
});

test("development preparation rejects an unknown profile before downloading build tools", () => {
  const result = spawnSync(
    process.execPath,
    [join(repository, "scripts/runtime/development.mjs"), "--profile", "unsupported"],
    { cwd: tmpdir(), encoding: "utf8", timeout: 10_000 },
  );
  assert.equal(result.error, undefined);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Unsupported Python runtime profile/u);
  assert.equal(result.stdout, "");
});
