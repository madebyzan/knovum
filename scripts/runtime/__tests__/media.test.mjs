import assert from "node:assert/strict";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { validateMediaLicenseReview } from "../media-license-policy.mjs";
import { mediaSource, validateMediaRuntime, writeMediaManifest } from "../media-manifest.mjs";
import { MediaRuntimeContentError } from "../media-runtime-content.mjs";

const repository = fileURLToPath(new URL("../../../", import.meta.url));

function fixture(context, target = "windows-x64") {
  const directory = mkdtempSync(join(tmpdir(), "knovum-core-media-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "bin"), { recursive: true });
  mkdirSync(join(directory, "build-evidence"));
  writeFileSync(join(directory, "bin/ffmpeg.exe"), "compiled executable");
  copyFileSync(
    join(repository, "scripts/runtime/media-license-review.json"),
    join(directory, "build-evidence/media-license-review.json"),
  );
  writeMediaManifest(repository, directory, target, {
    schemaVersion: 1,
    appCommit: "a".repeat(40),
    compiledBinaryFiles: ["bin/ffmpeg.exe"],
    target,
    version: "9.0.2",
    relocationVerified: true,
    signing: { mode: "unsigned", compiledBinaryCount: 1 },
    source: mediaSource(repository, target).source,
    licenseReview: validateMediaLicenseReview(repository, target, { requireApproved: false }),
  });
  return directory;
}

test("core media pins one static executable for each supported platform", () => {
  for (const target of ["macos-arm64", "macos-x64", "windows-x64"]) {
    const recipe = mediaSource(repository, target);
    assert.equal(recipe.version, "9.0.2");
    assert.match(recipe.source.sha256, /^[a-f0-9]{64}$/u);
    assert.ok(recipe.source.size > 0);
  }
});

test("media validation rejects corruption, injected files and a different supplier", (context) => {
  const root = fixture(context);
  validateMediaRuntime(repository, root, "windows-x64");
  const executable = join(root, "bin/ffmpeg.exe");
  writeFileSync(executable, "changed executable");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /file.*manifest/u);
  writeFileSync(executable, "compiled executable");
  writeFileSync(join(root, "python.exe"), "unexpected Python");
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), /undeclared/u);
  rmSync(join(root, "python.exe"));
  const path = join(root, "media-manifest.json");
  const manifest = JSON.parse(readFileSync(path));
  manifest.producer.source.sha256 = "b".repeat(64);
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
  symlinkSync("bin/ffmpeg.exe", join(root, "linked-ffmpeg"));
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), MediaRuntimeContentError);
});

test("media cache non-file executable entries are explicitly rebuildable", (context) => {
  const root = fixture(context);
  const executable = join(root, "bin/ffmpeg.exe");
  rmSync(executable);
  mkdirSync(executable);
  assert.throws(() => validateMediaRuntime(repository, root, "windows-x64"), MediaRuntimeContentError);
});

test("invalid static supplier entries are explicitly rebuildable", (context) => {
  const root = fixture(context);
  const path = join(root, "media-manifest.json");
  const manifest = JSON.parse(readFileSync(path));
  manifest.producer.source = null;
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
    chmodSync(join(restrictedRepository, "scripts/runtime/media-source.json"), 0o000);
    assert.throws(
      () => validateMediaRuntime(restrictedRepository, root, "windows-x64"),
      (error) => error?.code === "EACCES" && !(error instanceof MediaRuntimeContentError),
    );
  },
);

function repositoryWithConfiguration(context, configuration) {
  const directory = mkdtempSync(join(tmpdir(), "knovum-media-repository-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "scripts/runtime"), { recursive: true });
  writeFileSync(join(directory, "scripts/runtime/media-source.json"), configuration);
  return directory;
}
