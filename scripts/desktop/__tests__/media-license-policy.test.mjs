import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { mediaLegalSourceIdentity, validateMediaLicenseReview } from "../media-license-policy.mjs";

function fixture(context, reviewStatus = "pending") {
  const repository = mkdtempSync(join(tmpdir(), "knovum-media-license-review-"));
  context.after(() => rmSync(repository, { recursive: true, force: true }));
  const runtime = join(repository, "src-tauri/crates/media-native/native");
  mkdirSync(runtime, { recursive: true });
  const source = {
    schemaVersion: 1,
    version: "9.0.2",
    license: "GPL-3.0-or-later",
    licenseSource: { url: "https://license.example/GPL", sha256: "c".repeat(64) },
    sourceOffer: "https://source.example/ffmpeg-9.0.2.tar.xz",
    targets: {
      "macos-arm64": {
        sha256: "a".repeat(64),
        buildSource: "https://supplier.example/build-source",
        noticeSources: [{ url: "https://supplier.example/versions.txt", sha256: "e".repeat(64) }],
      },
    },
  };
  const approval = {
    legalSourceSha256: mediaLegalSourceIdentity(source, "macos-arm64"),
    reviewer: "release reviewer",
    reviewedAt: "2026-01-01T00:00:00Z",
    evidenceLocation: "https://evidence.example/reviews/ffmpeg-macos-arm64",
  };
  const review = {
    schemaVersion: 1,
    reviewStatus,
    version: source.version,
    targets: { "macos-arm64": reviewStatus === "approved" ? approval : null },
  };
  const save = () => {
    writeFileSync(join(runtime, "source.json"), `${JSON.stringify(source, null, 2)}\n`);
    writeFileSync(join(runtime, "license-review.json"), `${JSON.stringify(review, null, 2)}\n`);
  };
  save();
  return { repository, review, source, save };
}

test("pending media license review is allowed only for development", (context) => {
  const sample = fixture(context);
  assert.equal(
    validateMediaLicenseReview(sample.repository, "macos-arm64", { requireApproved: false }).status,
    "pending",
  );
  assert.throws(
    () => validateMediaLicenseReview(sample.repository, "macos-arm64"),
    /license review is unresolved/u,
  );
});

test("matching approved media license review allows production", (context) => {
  const sample = fixture(context, "approved");
  const receipt = validateMediaLicenseReview(sample.repository, "macos-arm64");
  assert.equal(receipt.status, "approved");
  assert.match(receipt.sha256, /^[a-f0-9]{64}$/u);
  assert.equal(receipt.legalSourceSha256, sample.review.targets["macos-arm64"].legalSourceSha256);
});

test("changed source archive invalidates an approved media review", (context) => {
  const sample = fixture(context, "approved");
  sample.source.targets["macos-arm64"].sha256 = "b".repeat(64);
  sample.save();
  assert.throws(
    () => validateMediaLicenseReview(sample.repository, "macos-arm64"),
    /different legal or source inputs/u,
  );
});

for (const [name, mutate] of [
  ["source offer", (source) => (source.sourceOffer = "https://source.example/changed.tar.xz")],
  ["license evidence", (source) => (source.licenseSource.sha256 = "d".repeat(64))],
  ["supplier build source", (source) => (source.targets["macos-arm64"].buildSource += "/changed")],
  ["supplier notices", (source) => (source.targets["macos-arm64"].noticeSources = [])],
]) {
  test(`changed ${name} invalidates an approved media review`, (context) => {
    const sample = fixture(context, "approved");
    mutate(sample.source);
    sample.save();
    assert.throws(
      () => validateMediaLicenseReview(sample.repository, "macos-arm64"),
      /different legal or source inputs/u,
    );
  });
}
