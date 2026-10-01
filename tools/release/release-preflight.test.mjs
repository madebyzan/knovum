import assert from "node:assert/strict";
import { existsSync, writeFileSync } from "node:fs";
import { test } from "node:test";

import { loadArtifacts } from "./artifacts.mjs";
import { createManifest } from "./manifest.mjs";
import { verifyVersions } from "./preflight.mjs";
import { signingConfiguration } from "./signing.mjs";
import { releaseFixture } from "./test-fixtures.mjs";
import { verifyHosted } from "./verify-hosted.mjs";

const versions = {
  tag: "v0.0.2",
  tauriVersion: "0.0.2",
  packageVersion: "0.0.2",
  cargo: { packages: [{ name: "knovum", version: "0.0.2" }] },
};

test("preflight requires all application versions to match the tag", () => {
  assert.equal(verifyVersions(versions), "0.0.2");
  for (const field of ["tauriVersion", "packageVersion"]) {
    assert.throws(() => verifyVersions({ ...versions, [field]: "0.0.1" }), /must match/);
  }
  assert.throws(() => verifyVersions({ ...versions, cargo: { packages: [] } }), /must match/);
  assert.throws(() => verifyVersions({ ...versions, tag: "main" }), /tag/);
  assert.throws(() => verifyVersions({ ...versions, tag: "v0.0.2-beta" }), /stable/);
});

test("macOS production configuration rejects ad-hoc signing or missing notarization", () => {
  const env = {
    APPLE_SIGNING_IDENTITY: "Developer ID Application: Example (TEAM)",
    APPLE_API_ISSUER: "issuer",
    APPLE_API_KEY: "key-id",
    APPLE_API_KEY_PATH: "key.p8",
  };
  assert.equal(signingConfiguration("macos-arm64", env).macOS.signingIdentity, env.APPLE_SIGNING_IDENTITY);
  assert.throws(
    () => signingConfiguration("macos-arm64", { ...env, APPLE_SIGNING_IDENTITY: "-" }),
    /Developer ID/,
  );
  assert.throws(() => signingConfiguration("macos-arm64", { ...env, APPLE_API_KEY_PATH: "" }), /required/);
});

test("Windows production configuration requires a certificate and secure RFC3161 timestamping", () => {
  const env = {
    KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT: "a".repeat(40),
    KNOVUM_WINDOWS_TIMESTAMP_URL: "https://timestamp.example.com",
  };
  const { windows } = signingConfiguration("windows-x64", env);
  assert.equal(windows.digestAlgorithm, "sha256");
  assert.equal(windows.tsp, true);
  assert.throws(() => signingConfiguration("windows-x64", {}), /required/);
  assert.throws(
    () => signingConfiguration("windows-x64", { ...env, KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT: "bad" }),
    /thumbprint/,
  );
  assert.throws(
    () =>
      signingConfiguration("windows-x64", { ...env, KNOVUM_WINDOWS_TIMESTAMP_URL: "http://insecure.test" }),
    /HTTPS/,
  );
  assert.throws(() => signingConfiguration("linux-x64", env), /Unsupported/);
});

function manifest(fixture) {
  return createManifest({
    version: fixture.version,
    notes: "notes",
    publishedAt: "2026-10-02T00:00:00Z",
    baseUrl: "https://cos.example.com/v0.0.2",
    artifacts: loadArtifacts(fixture.directory, fixture.version),
  });
}

test("host verification downloads every public URL with HTTPS-only redirects, then verifies signatures", (context) => {
  const fixture = releaseFixture(context);
  const files = [];
  const commands = [];
  verifyHosted(manifest(fixture), fixture.publicKey, (command, args) => {
    commands.push(command);
    if (command === "curl") {
      assert.equal(args[args.indexOf("--proto-redir") + 1], "=https");
      const file = args[args.indexOf("--output") + 1];
      files.push(file);
      writeFileSync(file, "test-only bytes, verifier is mocked");
    }
  });
  assert.deepEqual(commands, [
    "curl",
    "curl",
    "curl",
    "curl",
    "minisign",
    "minisign",
    "minisign",
    "minisign",
  ]);
  assert.ok(files.every((file) => !existsSync(file)));
});

test("host verification failure aborts publication and removes temporary files", (context) => {
  const fixture = releaseFixture(context);
  const files = [];
  assert.throws(
    () =>
      verifyHosted(manifest(fixture), fixture.publicKey, (command, args) => {
        if (command === "minisign") throw new Error("invalid signature");
        const file = args[args.indexOf("--output") + 1];
        files.push(file);
        writeFileSync(file, "test bytes");
      }),
    /invalid signature/,
  );
  assert.ok(files.every((file) => !existsSync(file)));
});
