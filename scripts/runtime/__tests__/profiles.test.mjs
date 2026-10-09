import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repository = fileURLToPath(new URL("../../../", import.meta.url)).replace(/\/$/u, "");

import {
  developmentDirectory,
  requirementsArguments,
  producerBytes,
  runtimeRecipeBytes,
  runtimeProfiles,
  runtimeTargetFromTriple,
} from "../profiles.mjs";

test("development identity binds the producer recipe as well as dependency locks", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-identity-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const name of ["python/runtimes/resources", "python/conda-locks/base", "scripts/runtime"])
    mkdirSync(join(root, name), { recursive: true });
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  cpSync(join(repository, "scripts/runtime"), join(root, "scripts/runtime"), { recursive: true });
  writeFileSync(join(root, "python/runtimes/resources/uv.lock"), "frozen dependencies");
  writeFileSync(join(root, "python/conda-locks/base/macos-x64.json"), "frozen Conda packages");
  writeFileSync(join(root, "scripts/runtime/conda_packages.py"), "first producer");
  const first = developmentDirectory(root, "macos-x64", "resources");
  assert.equal(developmentDirectory(root, "macos-x64", "resources"), first);
  writeFileSync(join(root, "scripts/runtime/conda_packages.py"), "corrected producer");
  assert.notEqual(developmentDirectory(root, "macos-x64", "resources"), first);
  writeFileSync(join(root, "scripts/runtime/conda_packages.py"), "first producer");
  writeFileSync(join(root, "scripts/runtime/provision.mjs"), "changed distribution consumer");
  writeFileSync(join(root, "scripts/runtime/unrelated.mjs"), "unrelated tool");
  assert.equal(developmentDirectory(root, "macos-x64", "resources"), first);
});

test("runtime recipe includes only the selected target and profile configuration", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-recipe-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "python/runtimes/resources"), { recursive: true });
  mkdirSync(join(root, "src-tauri"), { recursive: true });
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  copyFileSync(join(repository, "src-tauri/tauri.conf.json"), join(root, "src-tauri/tauri.conf.json"));
  const sourcePath = join(root, "python/runtime-source.json");
  const first = runtimeRecipeBytes(root, "macos-x64", "resources");
  const source = JSON.parse(readFileSync(sourcePath, "utf8"));
  source.uv.targets["windows-x64"].url = "https://example.invalid/unrelated-uv.zip";
  source.voiceLibrary.revision = "unrelated voice revision";
  source.uv.targets["macos-x64"] = Object.fromEntries(
    Object.entries(source.uv.targets["macos-x64"]).toReversed(),
  );
  writeFileSync(sourcePath, JSON.stringify(source, null, 4));
  assert.deepEqual(runtimeRecipeBytes(root, "macos-x64", "resources"), first);
  source.uv.targets["macos-x64"].url = "https://example.invalid/selected-uv.tar.gz";
  writeFileSync(sourcePath, JSON.stringify(source));
  assert.notDeepEqual(runtimeRecipeBytes(root, "macos-x64", "resources"), first);
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
  writeFileSync(join(root, "scripts/runtime/media_binary.py"), "changed static FFmpeg installer");
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

test("only supported desktop release triples can select a runtime", () => {
  assert.equal(runtimeTargetFromTriple("aarch64-apple-darwin"), "macos-arm64");
  assert.equal(runtimeTargetFromTriple("x86_64-pc-windows-msvc"), "windows-x64");
  assert.equal(runtimeTargetFromTriple(undefined), undefined);
  assert.equal(runtimeTargetFromTriple("x86_64-apple-darwin"), "macos-x64");
  assert.throws(() => runtimeTargetFromTriple("aarch64-pc-windows-msvc"), /Unsupported/);
});

test("development preparation rejects an unknown profile before downloading build tools", () => {
  const execution = spawnSync(
    process.execPath,
    [join(repository, "scripts/runtime/development.mjs"), "--profile", "unsupported"],
    { cwd: tmpdir(), encoding: "utf8", timeout: 10_000 },
  );
  assert.equal(execution.error, undefined);
  assert.notEqual(execution.status, 0);
  assert.match(execution.stderr, /Unsupported Python runtime profile/u);
  assert.equal(execution.stdout, "");
});
