import assert from "node:assert/strict";
import { test } from "node:test";

import { decodeBase64, httpsUrl, publicKey, releaseConfiguration, releaseRepository, stableVersion } from "./config.mjs";

// Structural fixture only. Never used to sign or enable a release.
const key = Buffer.concat([Buffer.from("Ed"), Buffer.alloc(40)]).toString("base64");
const encoded = Buffer.from(`untrusted comment: test public key\n${key}\n`).toString("base64");
const environment = {
  KNOVUM_UPDATER_ENDPOINT: "https://example.com/stable/latest.json",
  KNOVUM_UPDATER_PUBKEY: encoded,
  TAURI_SIGNING_PRIVATE_KEY: "test-only-private-key-path",
};

test("release configuration embeds the public key before plugin initialization", () => {
  const result = releaseConfiguration(environment);
  assert.deepEqual(result.plugins.updater.endpoints, [environment.KNOVUM_UPDATER_ENDPOINT]);
  assert.equal(result.plugins.updater.pubkey, encoded);
  assert.equal(result.plugins.updater.windows.installMode, "passive");
  assert.ok(!JSON.stringify(result).includes(environment.TAURI_SIGNING_PRIVATE_KEY));
  assert.deepEqual(publicKey(encoded), { encoded, minisign: key });
});

for (const name of Object.keys(environment)) {
  test(`release build rejects missing ${name}`, () => {
    const values = { ...environment };
    delete values[name];
    assert.throws(() => releaseConfiguration(values), new RegExp(name));
  });
}

for (const address of ["http://example.com/latest.json", "relative.json", "https://u:p@example.com/latest.json",
  "https://example.com/latest.json?token=secret", "https://example.com/latest.json#fragment",
  "https://example.com/{{target}}/{{arch}}"] ) {
  test(`rejects non-stable discovery address: ${address}`, () => {
    assert.throws(() => httpsUrl(address, "endpoint"));
  });
}

test("rejects malformed and private-key-shaped public configuration", () => {
  for (const value of ["not-a-key", key, "", Buffer.from("private key").toString("base64")]) {
    assert.throws(() => publicKey(value));
  }
  assert.throws(() => decodeBase64("YWJj!!", "fixture"));
});

test("stable release versions and repository selectors cannot inject paths or CLI flags", () => {
  assert.equal(stableVersion("0.0.2"), "0.0.2");
  for (const value of ["v1.2.3", "1.2.3-beta.1", "01.2.3", "1.2", "../../test", "1.2.3+build"]) {
    assert.throws(() => stableVersion(value));
  }
  assert.equal(releaseRepository("owner/knovum-releases"), "owner/knovum-releases");
  for (const value of ["--repo", "../repo", "owner/repo/extra", "https://github.com/owner/repo", "owner/.", "owner/.."]) {
    assert.throws(() => releaseRepository(value));
  }
});

test("release arguments cannot disable signing or override validated configuration", async () => {
  const { validateReleaseArguments } = await import("./config.mjs");
  for (const args of [["--debug"], ["-d"], ["--no-bundle"], ["--no-sign"], ["--config", "x.json"],
    ["--config=x.json"], ["-cx.json"], ["--features", "e2e"], ["--features=other,e2e"], ["-fe2e"]]) {
    assert.throws(() => validateReleaseArguments(args));
  }
  validateReleaseArguments(["--bundles", "app,dmg", "--target", "aarch64-apple-darwin"]);
});

test("release configuration rejects an inherited Tauri override", () => {
  assert.throws(
    () => releaseConfiguration({ ...environment, TAURI_CONFIG: '{"plugins":{"updater":{}}}' }),
    /TAURI_CONFIG override/,
  );
});
