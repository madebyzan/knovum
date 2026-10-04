import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { validateDependencyLicenseReview } from "../runtime-license-policy.mjs";
import { verifyRuntimeSignatures } from "../runtime-signatures.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

function fixture(context) {
  const repository = mkdtempSync(join(tmpdir(), "knovum-license-review-"));
  context.after(() => rmSync(repository, { recursive: true, force: true }));
  const python = join(repository, "python");
  const directory = join(repository, "runtime");
  mkdirSync(python);
  mkdirSync(join(directory, "build-evidence"), { recursive: true });
  writeFileSync(join(python, "uv.lock"), "locked graph");
  writeFileSync(join(directory, "build-evidence", "dependency-evidence.json"), "installed inventory");
  const review = {
    inventorySha256: hash("installed inventory"),
    reviewer: "release reviewer",
    reviewedAt: "2026-01-01T00:00:00Z",
    evidenceLocation: "https://example.test/reviews/runtime-1",
  };
  const policy = {
    schemaVersion: 1,
    reviewStatus: "approved",
    dependencyLockSha256: hash("locked graph"),
    targets: { "macos-arm64": review, "windows-x64": null },
  };
  return {
    repository,
    directory,
    review,
    policy,
    save: () => writeFileSync(join(python, "dependency-license-review.json"), JSON.stringify(policy)),
    validate: (target = "macos-arm64") => validateDependencyLicenseReview(directory, target, repository),
  };
}

test("approval is bound to the exact lock, conveyed inventory and target", (context) => {
  const sample = fixture(context);
  sample.save();
  assert.doesNotThrow(() => sample.validate());
  assert.throws(() => sample.validate("windows-x64"), /absent or invalid/);
  writeFileSync(join(sample.repository, "python", "uv.lock"), "changed graph");
  assert.throws(() => sample.validate(), /different lock/);
});

test("changed installed bytes invalidate a previously approved inventory", (context) => {
  const sample = fixture(context);
  sample.save();
  writeFileSync(join(sample.directory, "build-evidence", "dependency-evidence.json"), "changed inventory");
  assert.throws(() => sample.validate(), /conveyed inventory/);
});

test("production verification enforces unresolved review before invoking native tools", (context) => {
  const sample = fixture(context);
  sample.policy.reviewStatus = "required";
  sample.save();
  writeFileSync(
    join(sample.directory, "build-evidence", "producer.json"),
    JSON.stringify({
      signing: { mode: "production", identity: process.env.APPLE_SIGNING_IDENTITY },
    }),
  );
  assert.throws(
    () => verifyRuntimeSignatures(sample.directory, "macos-arm64", sample.repository),
    /license review is unresolved/,
  );
});

for (const [field, value, message] of [
  ["reviewer", " ", /accountable reviewer/],
  ["inventorySha256", "abc", /absent or invalid/],
  ["reviewedAt", "invalid", /completed review time/],
  ["reviewedAt", "2999-01-01T00:00:00Z", /completed review time/],
  ["evidenceLocation", "http://example.test/review", /durable HTTPS/],
  ["evidenceLocation", "https://user:password@example.test/review", /durable HTTPS/],
  ["evidenceLocation", "https://example.test/review#unbound", /durable HTTPS/],
]) {
  test(`approval rejects invalid ${field}: ${value}`, (context) => {
    const sample = fixture(context);
    sample.review[field] = value;
    sample.save();
    assert.throws(() => sample.validate(), message);
  });
}
