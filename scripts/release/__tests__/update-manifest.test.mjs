import assert from "node:assert/strict";
import { test } from "node:test";

import { releaseTargets } from "../policy.mjs";
import { assertPromotion, createManifest, validateManifest } from "../update-manifest.mjs";

const signature = Buffer.from(
  "untrusted comment: test signature\nnot-used-for-cryptographic-verification\n",
).toString("base64");
const artifacts = Object.values(releaseTargets).map((target) => ({
  version: "1.2.3",
  target,
  file: `Knovum_1.2.3_${target}.bin`,
  signature,
}));
const options = {
  version: "1.2.3",
  notes: "Reviewed release notes",
  pubDate: "2026-10-02T00:00:00Z",
  artifacts,
  baseUrl: "https://github.com/example/distribution/releases/download/v1.2.3",
};

test("release policy contains only the approved targets", () => {
  assert.deepEqual(Object.keys(releaseTargets), [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
  ]);
  assert.throws(() =>
    createManifest({
      ...options,
      artifacts: [...artifacts, { ...artifacts[0], target: "windows-aarch64" }],
    }),
  );
});

test("standard Tauri manifests separate the permanent discovery endpoint from artifact hosting", () => {
  const github = createManifest(options);
  const cos = createManifest({ ...options, baseUrl: "https://downloads.example.test/knovum/1.2.3" });
  assertPromotion(github, cos);
  for (const target of Object.values(releaseTargets)) {
    assert.equal(github.platforms[target].signature, cos.platforms[target].signature);
    assert.ok(cos.platforms[target].url.startsWith("https://downloads.example.test/knovum/1.2.3/"));
  }
});

test("partial, duplicate, mixed-version and unsafe artifact sets are rejected", () => {
  for (const items of [
    artifacts.slice(1),
    [...artifacts, artifacts[0]],
    artifacts.map((item, index) => (index === 0 ? { ...item, version: "1.2.4" } : item)),
    artifacts.map((item) => ({ ...item, file: "../escape" })),
  ]) {
    assert.throws(() => createManifest({ ...options, artifacts: items }));
  }
});

test("existing signed versions cannot be overwritten or downgraded", () => {
  const previous = createManifest(options);
  const changed = structuredClone(previous);
  changed.platforms[artifacts[0].target].signature = Buffer.from("untrusted comment: replacement\n").toString(
    "base64",
  );
  assert.throws(() => assertPromotion(previous, changed), /immutable/);
  assert.throws(() => assertPromotion(previous, { ...previous, version: "1.2.2" }), /downgrade/);
  assert.throws(() => assertPromotion(previous, { ...previous, notes: "Changed" }), /URLs only/);
  assert.doesNotThrow(() => assertPromotion(previous, { ...previous, version: "1.2.4" }));
});

test("empty notes, invalid dates and insecure download URLs fail validation", () => {
  const manifest = createManifest(options);
  assert.throws(() => validateManifest({ ...manifest, notes: " " }));
  assert.throws(() => validateManifest({ ...manifest, pub_date: "yesterday" }));
  manifest.platforms[artifacts[0].target].url = "http://example.test/update";
  assert.throws(() => validateManifest(manifest));
});
