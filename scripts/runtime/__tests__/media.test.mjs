import assert from "node:assert/strict";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { sha256File } from "../files.mjs";
import { validateMediaRuntime, writeMediaManifest } from "../media-manifest.mjs";
import { MediaRuntimeContentError } from "../media-runtime-content.mjs";

const repository = fileURLToPath(new URL("../../../", import.meta.url));

function fixture(context, target = "windows-x64") {
  const directory = mkdtempSync(join(tmpdir(), "knovum-core-media-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "Library/bin"), { recursive: true });
  writeFileSync(join(directory, "Library/bin/ffmpeg.exe"), "compiled executable");
  const lock = JSON.parse(readFileSync(join(repository, "python/conda-locks/media", `${target}.json`)));
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json")));
  writeMediaManifest(repository, directory, target, {
    schemaVersion: 1,
    appCommit: "a".repeat(40),
    compiledDependencyFiles: ["Library/bin/ffmpeg.exe"],
    target,
    version: "9.0.1",
    relocationVerified: true,
    signing: { mode: "unsigned", compiledBinaryCount: 1 },
    condaSupplier: {
      supplier: "conda-forge",
      micromambaVersion: configuration.condaSupplier.micromambaVersion,
      lockSha256: sha256File(join(repository, "python/conda-locks/media", `${target}.json`)),
      packages: lock.packages.map((p) => Object.assign({}, p, { files: [] })),
    },
  });
  return directory;
}

test("core media has a frozen Conda closure on all supported platforms without Python", () => {
  for (const target of ["macos-arm64", "macos-x64", "windows-x64"]) {
    const lock = JSON.parse(readFileSync(join(repository, "python/conda-locks/media", `${target}.json`)));
    assert.equal(lock.packages.find((p) => p.name === "ffmpeg").version, "9.0.1");
    assert.equal(
      lock.packages.some((p) => p.name === "python"),
      false,
    );
    assert.ok(lock.packages.every((p) => /^[a-f0-9]{64}$/u.test(p.sha256)));
  }
});

test("media validation rejects corruption, injected files and a different Conda closure", (context) => {
  const root = fixture(context);
  validateMediaRuntime(repository, root, "windows-x64");
  const executable = join(root, "Library/bin/ffmpeg.exe");
  writeFileSync(executable, "changed executable");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /file.*manifest/u);
  writeFileSync(executable, "compiled executable");
  writeFileSync(join(root, "python.exe"), "unexpected Python");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /undeclared/u);
  rmSync(join(root, "python.exe"));
  const path = join(root, "media-manifest.json");
  const manifest = JSON.parse(readFileSync(path));
  manifest.producer.condaSupplier.packages[0].sha256 = "b".repeat(64);
  writeFileSync(path, JSON.stringify(manifest));
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /controlled build/u);
});

test("media validation rejects another target and an obsolete producer recipe", (context) => {
  const root = fixture(context);
  assert.throws(() => validateMediaRuntime(repository, root, "macos-x64"), /identity/u);
  const path = join(root, "media-manifest.json");
  const manifest = JSON.parse(readFileSync(path));
  manifest.recipeSha256 = "b".repeat(64);
  writeFileSync(path, JSON.stringify(manifest));
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /identity/u);
});

test("media cache content failures are explicitly rebuildable", (context) => {
  const root = fixture(context);
  writeFileSync(join(root, "media-manifest.json"), "{");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), MediaRuntimeContentError);
});

test("media cache links are explicitly rebuildable", { skip: process.platform === "win32" }, (context) => {
  const root = fixture(context);
  symlinkSync("Library/bin/ffmpeg.exe", join(root, "linked-ffmpeg"));
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), MediaRuntimeContentError);
});

test("media cache non-file executable entries are explicitly rebuildable", (context) => {
  const root = fixture(context);
  const executable = join(root, "Library/bin/ffmpeg.exe");
  rmSync(executable);
  mkdirSync(executable);
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), MediaRuntimeContentError);
});

test("invalid Conda supplier package entries are explicitly rebuildable", (context) => {
  const root = fixture(context);
  const path = join(root, "media-manifest.json");
  const manifest = JSON.parse(readFileSync(path));
  manifest.producer.condaSupplier.packages = [null];
  writeFileSync(path, JSON.stringify(manifest));
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), MediaRuntimeContentError);
});

test("repository configuration syntax and validator type errors are not cache failures", (context) => {
  const root = fixture(context);
  const invalidJsonRepository = repositoryWithConfiguration(context, "{");
  assert.throws(
    () => validateMediaRuntime(invalidJsonRepository, root, "windows-x64"),
    (error) => error instanceof SyntaxError && !(error instanceof MediaRuntimeContentError),
  );

  const invalidShapeRepository = repositoryWithConfiguration(context, "{}");
  assert.throws(
    () => validateMediaRuntime(invalidShapeRepository, root, "windows-x64"),
    (error) => error instanceof TypeError && !(error instanceof MediaRuntimeContentError),
  );
});

test(
  "repository configuration permission errors are not cache failures",
  { skip: process.platform === "win32" },
  (context) => {
    const root = fixture(context);
    const restrictedRepository = repositoryWithConfiguration(context, "{}");
    chmodSync(join(restrictedRepository, "python/runtime-source.json"), 0o000);
    assert.throws(
      () => validateMediaRuntime(restrictedRepository, root, "windows-x64"),
      (error) => error?.code === "EACCES" && !(error instanceof MediaRuntimeContentError),
    );
  },
);

function repositoryWithConfiguration(context, configuration) {
  const directory = mkdtempSync(join(tmpdir(), "knovum-media-repository-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "python"), { recursive: true });
  writeFileSync(join(directory, "python/runtime-source.json"), configuration);
  return directory;
}
