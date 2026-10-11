import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repository = fileURLToPath(new URL("../../../", import.meta.url)).replace(/\/$/u, "");

import { mediaRecipeIdentity } from "../media-manifest.mjs";
import {
  developmentDirectory,
  requirementsArguments,
  producerBytes,
  runtimeRecipeBytes,
  runtimeTargetFromTriple,
} from "../profiles.mjs";

test("development identity binds the producer recipe as well as dependency locks", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-identity-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const name of ["python/runtimes/agent", "python/conda-locks/base", "scripts/runtime"])
    mkdirSync(join(root, name), { recursive: true });
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  cpSync(join(repository, "scripts/runtime"), join(root, "scripts/runtime"), { recursive: true });
  writeFileSync(join(root, "python/runtimes/agent/uv.lock"), "frozen dependencies");
  writeFileSync(join(root, "python/conda-locks/base/macos-x64.json"), "frozen Conda packages");
  writeFileSync(join(root, "scripts/runtime/conda_packages.py"), "first producer");
  const first = developmentDirectory(root, "macos-x64", "agent");
  assert.equal(developmentDirectory(root, "macos-x64", "agent"), first);
  writeFileSync(join(root, "scripts/runtime/conda_packages.py"), "corrected producer");
  assert.notEqual(developmentDirectory(root, "macos-x64", "agent"), first);
  writeFileSync(join(root, "scripts/runtime/conda_packages.py"), "first producer");
  writeFileSync(join(root, "scripts/runtime/provision.mjs"), "changed distribution consumer");
  writeFileSync(join(root, "scripts/runtime/unrelated.mjs"), "unrelated tool");
  assert.equal(developmentDirectory(root, "macos-x64", "agent"), first);
});

test("runtime recipe includes only the selected target and profile configuration", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-recipe-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "python/runtimes/agent"), { recursive: true });
  mkdirSync(join(root, "src-tauri"), { recursive: true });
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  copyFileSync(join(repository, "src-tauri/tauri.conf.json"), join(root, "src-tauri/tauri.conf.json"));
  const sourcePath = join(root, "python/runtime-source.json");
  const first = runtimeRecipeBytes(root, "macos-x64", "agent");
  const source = JSON.parse(readFileSync(sourcePath, "utf8"));
  source.uv.targets["windows-x64"].url = "https://example.invalid/unrelated-uv.zip";
  source.uv.targets["macos-x64"] = Object.fromEntries(
    Object.entries(source.uv.targets["macos-x64"]).toReversed(),
  );
  writeFileSync(sourcePath, JSON.stringify(source, null, 4));
  assert.deepEqual(runtimeRecipeBytes(root, "macos-x64", "agent"), first);
  source.uv.targets["macos-x64"].url = "https://example.invalid/selected-uv.tar.gz";
  writeFileSync(sourcePath, JSON.stringify(source));
  assert.notDeepEqual(runtimeRecipeBytes(root, "macos-x64", "agent"), first);
});

test("producer identity follows the selected resource's actual source closure", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-producer-scope-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "python"));
  copyFileSync(join(repository, "python/runtime-source.json"), join(root, "python/runtime-source.json"));
  cpSync(join(repository, "scripts/runtime"), join(root, "scripts/runtime"), { recursive: true });
  const before = producerBytes(root, "agent"),
    mediaBefore = mediaRecipeIdentity(root, "macos-x64");
  writeFileSync(join(root, "scripts/runtime/voice_bridge.cpp"), "changed voice supplier");
  assert(producerBytes(root, "agent").equals(before));
  assert.equal(mediaRecipeIdentity(root, "macos-x64"), mediaBefore);
  writeFileSync(join(root, "scripts/runtime/media-binary.mjs"), "changed static FFmpeg installer");
  assert.notEqual(mediaRecipeIdentity(root, "macos-x64"), mediaBefore);
  assert(producerBytes(root, "agent").equals(before));
  assert.throws(() => producerBytes(root, "media"), /Unsupported/u);
});

test("Agent Runtime requirements select their independent project and exclude inference extras", () => {
  const args = requirementsArguments(repository, "agent");
  assert.equal(args[args.indexOf("--project") + 1], `${repository}/python/runtimes/agent`);
  assert.equal(args.includes("--extra"), false);
  assert.throws(() => requirementsArguments(repository, "unknown"), /Unsupported/);
});

test("requirements exclusions follow the configured profile group", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-groups-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "python"));
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  configuration.condaSupplier.profiles.agent = "alternate";
  configuration.condaSupplier.pythonDistributions.alternate = ["owned-exclusion"];
  writeFileSync(join(root, "python/runtime-source.json"), JSON.stringify(configuration));
  const args = requirementsArguments(root, "agent");
  const exclusions = args.flatMap((value, index) => (value === "--no-emit-package" ? [args[index + 1]] : []));
  assert.deepEqual(exclusions, configuration.condaSupplier.pythonDistributions.alternate);
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
