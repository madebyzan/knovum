import { readFileSync } from "node:fs";
import { join } from "node:path";

import { sha256File } from "./files.mjs";

export function validateDependencyLicenseReview(directory, target, repository) {
  const policy = JSON.parse(
    readFileSync(join(repository, "python", "dependency-license-review.json"), "utf8"),
  );
  if (policy.schemaVersion !== 1 || policy.reviewStatus !== "approved") {
    throw new Error("Python dependency license review is unresolved; production packaging is blocked");
  }
  if (policy.dependencyLockSha256 !== sha256File(join(repository, "python", "uv.lock"))) {
    throw new Error("Python dependency license review belongs to a different lock");
  }
  const review = policy.targets?.[target];
  validateReview(review);
  const inventory = join(directory, "build-evidence", "dependency-evidence.json");
  if (review.inventorySha256 !== sha256File(inventory)) {
    throw new Error("Python dependency license review does not match the conveyed inventory");
  }
}

function validateReview(review) {
  if (!review || !/^[a-f0-9]{64}$/u.test(review.inventorySha256 ?? "")) {
    throw new Error("Python target license review is absent or invalid");
  }
  if (typeof review.reviewer !== "string" || !review.reviewer.trim()) {
    throw new Error("Python license review requires an accountable reviewer");
  }
  const timestamp = Date.parse(review.reviewedAt);
  if (!Number.isFinite(timestamp) || timestamp > Date.now()) {
    throw new Error("Python license review requires a valid completed review time");
  }
  validateEvidenceLocation(review.evidenceLocation);
}

function validateEvidenceLocation(location) {
  const evidence = new URL(location);
  if (evidence.protocol !== "https:" || evidence.username || evidence.password || evidence.hash) {
    throw new Error("Python license review requires a durable HTTPS evidence location");
  }
}
