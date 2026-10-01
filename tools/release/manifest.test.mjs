import assert from "node:assert/strict";
import { test } from "node:test";

import {
  artifactName,
  createManifest,
  relocateManifest,
  releaseTargets,
  signatureText,
} from "./manifest.mjs";

// This is not a cryptographic verification fixture. Publishing independently runs minisign.
const packet = Buffer.concat([Buffer.from("ED"), Buffer.alloc(72)]).toString("base64");
const globalSignature = Buffer.alloc(64).toString("base64");
const signature = Buffer.from(
  `untrusted comment: fixture\n${packet}\ntrusted comment: fixture\n${globalSignature}\n`,
).toString("base64");
const fixture = () => ({
  version: "0.0.2",
  notes: "Fix updater initialization.",
  publishedAt: "2026-10-02T00:00:00Z",
  baseUrl: "https://github.com/owner/releases/releases/download/v0.0.2",
  artifacts: releaseTargets.map((target) => ({ target, name: artifactName("0.0.2", target), signature })),
});

test("creates a complete, platform-specific Tauri static manifest", () => {
  const manifest = createManifest(fixture());
  assert.deepEqual(Object.keys(manifest.platforms), releaseTargets);
  assert.equal(manifest.version, "0.0.2");
  assert.equal(manifest.pub_date, "2026-10-02T00:00:00.000Z");
  assert.ok(manifest.platforms["windows-x86_64"].url.endsWith("-setup.exe"));
  assert.notEqual(manifest.platforms["darwin-aarch64"].url, manifest.platforms["darwin-x86_64"].url);
});

test("GitHub to COS migration changes URLs only, preserving all signed metadata", () => {
  const before = createManifest(fixture());
  const after = relocateManifest(before, "https://example.cos.ap-tokyo.myqcloud.com/v0.0.2");
  assert.equal(after.version, before.version);
  assert.equal(after.notes, before.notes);
  assert.equal(after.pub_date, before.pub_date);
  for (const target of releaseTargets) {
    assert.equal(after.platforms[target].signature, before.platforms[target].signature);
    assert.ok(after.platforms[target].url.startsWith("https://example.cos.ap-tokyo.myqcloud.com/v0.0.2/"));
  }
  assert.ok(before.platforms[releaseTargets[0]].url.startsWith("https://github.com/"));
});

test("rejects incomplete, duplicate, and unsupported release matrices", () => {
  const missing = fixture();
  missing.artifacts.pop();
  assert.throws(() => createManifest(missing), /incomplete/);
  const duplicate = fixture();
  duplicate.artifacts[2] = duplicate.artifacts[0];
  assert.throws(() => createManifest(duplicate), /Duplicate/);
  assert.throws(() => artifactName("0.0.2", "linux-x86_64"), /Unsupported/);
});

test("rejects mutable or traversal filenames and invalid metadata", () => {
  const input = fixture();
  input.artifacts[0].name = "../../Knovum.app.tar.gz";
  assert.throws(() => createManifest(input), /filename/);
  assert.throws(() => createManifest({ ...fixture(), notes: "" }), /notes/);
  assert.throws(() => createManifest({ ...fixture(), publishedAt: "not-a-date" }), /date/);
  assert.throws(() => relocateManifest(createManifest(fixture()), "http://example.com/releases"), /HTTPS/);
});

test("rejects signature URLs, incomplete envelopes, and malformed base64", () => {
  assert.equal(signatureText(signature).encoded, signature);
  for (const value of [
    "https://example.com/signature.sig",
    "artifact.sig",
    "invalid",
    Buffer.from("signature").toString("base64"),
  ]) {
    assert.throws(() => signatureText(value));
  }
});

test("rejects impossible calendar dates rather than normalizing them", () => {
  for (const publishedAt of ["2026-02-30T00:00:00Z", "2026-10-02T24:00:00Z", "2026-10-02"]) {
    assert.throws(() => createManifest({ ...fixture(), publishedAt }), /date/i);
  }
  assert.equal(
    createManifest({ ...fixture(), publishedAt: "2026-10-02T00:00:00.1Z" }).pub_date,
    "2026-10-02T00:00:00.100Z",
  );
});
