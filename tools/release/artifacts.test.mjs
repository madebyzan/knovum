import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { loadArtifacts, stageTarget, verifyArtifacts } from "./artifacts.mjs";
import { artifactName, releaseTargets } from "./manifest.mjs";
import { releaseFixture, testSignature } from "./test-fixtures.mjs";

test("all targets must exist before verification", (context) => {
  const fixture = releaseFixture(context);
  assert.equal(loadArtifacts(fixture.directory, fixture.version).length, releaseTargets.length);
  rmSync(join(fixture.directory, artifactName(fixture.version, "windows-x86_64")));
  assert.throws(() => loadArtifacts(fixture.directory, fixture.version));
});

test("symbolic links are not accepted as release artifacts", (context) => {
  const fixture = releaseFixture(context);
  const path = join(fixture.directory, artifactName(fixture.version, "darwin-aarch64"));
  rmSync(path);
  symlinkSync(fixture.notesFile, path);
  assert.throws(() => loadArtifacts(fixture.directory, fixture.version), /regular/);
});

test("verification delegates each artifact to minisign and removes temporary signatures", (context) => {
  const fixture = releaseFixture(context);
  const signatures = [];
  verifyArtifacts(fixture.directory, loadArtifacts(fixture.directory, fixture.version), fixture.publicKey, (command, args) => {
    assert.equal(command, "minisign");
    assert.equal(args[0], "-Vm");
    assert.equal(args[2], "-P");
    assert.equal(args[4], "-x");
    signatures.push(args[5]);
    assert.match(readFileSync(args[5], "utf8"), /^untrusted comment:/);
  });
  assert.equal(signatures.length, releaseTargets.length);
  assert.ok(signatures.every((path) => !existsSync(path)));
});

test("a signature failure aborts the sequence and cleans up", (context) => {
  const fixture = releaseFixture(context);
  let signatureFile;
  let count = 0;
  assert.throws(() => verifyArtifacts(fixture.directory, loadArtifacts(fixture.directory, fixture.version), fixture.publicKey, (_, args) => {
    count += 1;
    signatureFile = args[5];
    throw new Error("signature rejected");
  }), /rejected/);
  assert.equal(count, 1);
  assert.equal(existsSync(signatureFile), false);
});

test("staging gives architectures distinct immutable filenames", (context) => {
  const fixture = releaseFixture(context);
  const bundleDirectory = join(fixture.directory, "bundle");
  mkdirSync(join(bundleDirectory, "macos"), { recursive: true });
  mkdirSync(join(bundleDirectory, "dmg"), { recursive: true });
  writeFileSync(join(bundleDirectory, "dmg/Knovum.dmg"), "signed-dmg-fixture");
  writeFileSync(join(bundleDirectory, "macos/Knovum.app.tar.gz"), "archive");
  writeFileSync(join(bundleDirectory, "macos/Knovum.app.tar.gz.sig"), testSignature);
  const outputDirectory = join(fixture.directory, "staged");
  const options = { bundleDirectory, outputDirectory, version: fixture.version, target: "darwin-aarch64" };
  stageTarget(options);
  const path = join(outputDirectory, artifactName(fixture.version, options.target));
  assert.equal(readFileSync(path, "utf8"), "archive");
  assert.throws(() => stageTarget(options));
  assert.equal(readFileSync(path, "utf8"), "archive");
});
