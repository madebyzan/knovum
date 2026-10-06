import { readFileSync } from "node:fs";
import { join } from "node:path";

import { sha256File } from "./files.mjs";
import { runtimeProject } from "./profiles.mjs";

export function validateDependencyLicenseReview(directory, target, repository, profile) {
  const policy = JSON.parse(
    readFileSync(join(runtimeProject(repository, profile), "dependency-license-review.json"), "utf8"),
  );
  if (policy.schemaVersion !== 1 || policy.reviewStatus !== "approved") {
    throw new Error("Python dependency license review is unresolved; production packaging is blocked");
  }
  const manifest = JSON.parse(readFileSync(join(directory, "runtime-manifest.json"), "utf8"));
  if (
    policy.dependencyRequirementsSha256 !== manifest.dependencyRequirementsSha256 ||
    policy.dependencyLockSha256 !== manifest.dependencyLockSha256 ||
    policy.profile !== manifest.profile
  ) {
    throw new Error("Python dependency license review belongs to different runtime dependencies");
  }
  const review = policy.targets?.[target];
  validateReview(review);
  const inventory = join(directory, "..", "build-evidence", "dependency-evidence.json");
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
