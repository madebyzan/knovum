import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  compareVersions,
  distributionRepository,
  httpsUrl,
  platformSigningConfiguration,
  publicKey,
  readVersion,
  stableVersion,
  updaterConfiguration,
  validateReleaseArguments,
} from "../policy.mjs";

const record = Buffer.alloc(42);
record.write("Ed");
const key = Buffer.from(`untrusted comment: test public key\n${record.toString("base64")}\n`).toString(
  "base64",
);
const valid = {
  KNOVUM_UPDATER_ENDPOINT: "https://updates.example.test/stable.json",
  KNOVUM_UPDATER_PUBKEY: key,
  TAURI_SIGNING_PRIVATE_KEY: "test-only-private-key-path",
};

test("release configuration embeds public verification configuration, never signing secrets", () => {
  const config = updaterConfiguration(valid);
  assert.deepEqual(config, {
    pubkey: key,
    endpoints: [valid.KNOVUM_UPDATER_ENDPOINT],
    windows: { installMode: "passive" },
  });
  assert.ok(!JSON.stringify(config).includes(valid.TAURI_SIGNING_PRIVATE_KEY));
});

test("all release credentials/configuration are required; no disabled release fallback", () => {
  for (const name of Object.keys(valid))
    assert.throws(() => updaterConfiguration({ ...valid, [name]: " " }), /required/);
});

test("updater URLs cannot be insecure, credential-bearing or temporary signed URLs", () => {
  for (const url of [
    "http://example.test/u.json",
    "https://user:secret@example.test/u.json",
    "https://example.test/u.json?token=secret",
    "https://example.test/u.json#fragment",
    "file:///tmp/u.json",
  ]) {
    assert.throws(() => httpsUrl(url, "endpoint"));
  }
  assert.equal(
    httpsUrl("https://raw.githubusercontent.com/a/b/main/channels/stable.json", "endpoint").protocol,
    "https:",
  );
});

test("public key validation rejects paths, malformed encodings and non-key material", () => {
  for (const value of ["/tmp/key.pub", "not-base64", "", Buffer.from("a private key").toString("base64")])
    assert.throws(() => publicKey(value));
  assert.equal(publicKey(key), key);
});

test("stable versions sort numerically without precision loss and exclude prereleases", () => {
  assert.equal(compareVersions("1.10.0", "1.9.9"), 1);
  assert.equal(compareVersions("9007199254740993.0.0", "9007199254740992.0.0"), 1);
  assert.equal(compareVersions("1.2.3", "1.2.3"), 0);
  for (const version of ["v1.0.0", "1.0.0-beta", "01.0.0", "1.0", "1.0.0+build"])
    assert.throws(() => stableVersion(version));
});

test("release metadata has one aligned version source", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-release-"));
  try {
    mkdirSync(join(root, "src-tauri"));
    writeFileSync(join(root, "package.json"), JSON.stringify({ version: "1.2.3" }));
    writeFileSync(join(root, "src-tauri/tauri.conf.json"), JSON.stringify({ version: "1.2.3" }));
    writeFileSync(
      join(root, "src-tauri/Cargo.toml"),
      '[package]\nname = "knovum"\nversion = "1.2.3"\n[dependencies]\n',
    );
    assert.equal(readVersion(root), "1.2.3");
    writeFileSync(join(root, "package.json"), JSON.stringify({ version: "1.2.4" }));
    assert.throws(() => readVersion(root), /must match/);
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("release builds reject switches that bypass signing or inject unrelated configuration", () => {
  validateReleaseArguments(["--target", "aarch64-apple-darwin", "--bundles", "app,dmg"]);
  for (const args of [
    ["--debug"],
    ["--config", "other.json"],
    ["--no-sign"],
    ["--target"],
    ["--bundles", "app", "--bundles", "dmg"],
  ]) {
    assert.throws(() => validateReleaseArguments(args));
  }
});

test("production packaging requires an explicit platform signing policy", () => {
  assert.throws(
    () => platformSigningConfiguration("macos-arm64", { APPLE_SIGNING_IDENTITY: "-" }),
    /Developer ID/,
  );
  assert.throws(() => platformSigningConfiguration("windows-x64", {}), /KNOVUM_WINDOWS_SIGN_COMMAND/);
  assert.throws(() => platformSigningConfiguration("linux-x64", {}), /Unsupported/);
  assert.throws(() => platformSigningConfiguration("macos-x64", {}), /APPLE_SIGNING_IDENTITY/);
  assert.throws(() => platformSigningConfiguration("windows-arm64", {}), /Unsupported/);
  const env = {
    APPLE_SIGNING_IDENTITY: "Developer ID Application: Test",
    APPLE_API_KEY: "key",
    APPLE_API_ISSUER: "issuer",
    APPLE_API_KEY_PATH: "/tmp/key.p8",
  };
  assert.deepEqual(platformSigningConfiguration("macos-arm64", env), {});
});

test("distribution repository configuration cannot inject API paths", () => {
  assert.equal(distributionRepository("madebyzan/knovum-releases"), "madebyzan/knovum-releases");
  for (const value of [
    "../releases",
    "owner/../../secrets",
    "owner/repo?token=x",
    "https://github.com/owner/repo",
  ]) {
    assert.throws(() => distributionRepository(value));
  }
});
