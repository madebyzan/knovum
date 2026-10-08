import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { sha256File } from "../files.mjs";
import { validateMediaRuntime, writeMediaManifest } from "../media-manifest.mjs";

const repository = fileURLToPath(new URL("../../../", import.meta.url));

function fixture(context, target = "windows-x64") {
  const directory = mkdtempSync(join(tmpdir(), "knovum-core-media-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "Library/bin"), { recursive: true });
  writeFileSync(join(directory, "Library/bin/ffmpeg.exe"), "native executable");
  const lock = JSON.parse(readFileSync(join(repository, "python/native-locks/media", `${target}.json`)));
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json")));
  writeMediaManifest(repository, directory, target, {
    schemaVersion: 1,
    appCommit: "a".repeat(40),
    nativeDependencyFiles: ["Library/bin/ffmpeg.exe"],
    target,
    version: "9.0.1",
    relocationVerified: true,
    signing: { mode: "unsigned", compiledBinaryCount: 1 },
    nativeSupplier: {
      supplier: "conda-forge",
      micromambaVersion: configuration.nativeSupplier.micromambaVersion,
      lockSha256: sha256File(join(repository, "python/native-locks/media", `${target}.json`)),
      packages: lock.packages.map((p) => Object.assign({}, p, { files: [] })),
    },
  });
  return directory;
}

test("core media has a frozen native closure on all supported platforms without Python", () => {
  for (const target of ["macos-arm64", "macos-x64", "windows-x64"]) {
    const lock = JSON.parse(readFileSync(join(repository, "python/native-locks/media", `${target}.json`)));
    assert.equal(lock.packages.find((p) => p.name === "ffmpeg").version, "9.0.1");
    assert.equal(
      lock.packages.some((p) => p.name === "python"),
      false,
    );
    assert.ok(lock.packages.every((p) => /^[a-f0-9]{64}$/u.test(p.sha256)));
  }
});

test("media validation rejects corruption, injected files and a different native closure", (context) => {
  const root = fixture(context);
  validateMediaRuntime(repository, root, "windows-x64");
  const executable = join(root, "Library/bin/ffmpeg.exe");
  writeFileSync(executable, "changed executable");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /file.*manifest/u);
  writeFileSync(executable, "native executable");
  writeFileSync(join(root, "python.exe"), "unexpected Python");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /undeclared/u);
  rmSync(join(root, "python.exe"));
  const path = join(root, "media-manifest.json");
  const manifest = JSON.parse(readFileSync(path));
  manifest.producer.nativeSupplier.packages[0].sha256 = "b".repeat(64);
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
