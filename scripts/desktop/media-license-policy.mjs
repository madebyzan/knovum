import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

function canonical(entry) {
  if (Array.isArray(entry)) return entry.map(canonical);
  if (entry !== null && typeof entry === "object")
    return Object.fromEntries(
      Object.keys(entry)
        .toSorted((a, b) => a.localeCompare(b, "en"))
        .map((key) => [key, canonical(entry[key])]),
    );
  return entry;
}
function recipeBytes(value) {
  return Buffer.from(JSON.stringify(canonical(value)));
}

const reviewName = "license-review.json";

export function validateMediaLicenseReview(repository, target, { requireApproved = true } = {}) {
  const source = readJson(join(repository, "src-tauri/crates/media-libraries/cmake/source.json"));
  const reviewPath = join(repository, "src-tauri/crates/media-libraries/cmake", reviewName);
  const reviewBytes = readFileSync(reviewPath);
  const review = JSON.parse(reviewBytes);
  const targets = validatePolicy(review, source, target);
  if (review.reviewStatus === "pending") validatePending(review, requireApproved);
  else validateApprovals(review, source, targets, target);

  return {
    status: review.reviewStatus,
    sha256: createHash("sha256").update(reviewBytes).digest("hex"),
    legalSourceSha256: mediaLegalSourceIdentity(source, target),
  };
}

export function mediaLegalSourceIdentity(source, target) {
  return createHash("sha256")
    .update(
      recipeBytes({
        schemaVersion: source.schemaVersion,
        version: source.version,
        license: source.license,
        licenseSource: source.licenseSource,
        sourceOffer: source.sourceOffer,
        target,
        source: source.targets[target],
      }),
    )
    .digest("hex");
}

function validatePolicy(review, source, target) {
  const targets = Object.keys(source.targets ?? {}).toSorted();
  if (
    !exactKeys(review, ["schemaVersion", "reviewStatus", "targets", "version"]) ||
    review.schemaVersion !== 1 ||
    review.version !== source.version ||
    !exactKeys(review.targets, targets) ||
    !["approved", "pending"].includes(review.reviewStatus) ||
    !source.targets?.[target]
  )
    throw new Error("Invalid core media license review policy");
  return targets;
}

function validatePending(review, requireApproved) {
  if (Object.values(review.targets).some((entry) => entry !== null))
    throw new Error("Invalid pending core media license review policy");
  if (requireApproved)
    throw new Error("Core media license review is unresolved; release packaging is blocked");
}

function validateApprovals(review, source, targets, target) {
  for (const reviewedTarget of targets) {
    const entry = review.targets[reviewedTarget];
    if (entry !== null) validateApproval(entry, source, reviewedTarget);
  }
  if (review.targets[target] === null) throw new Error(`Core media license review is absent for ${target}`);
}

function validateApproval(approval, source, target) {
  if (!exactKeys(approval, ["evidenceLocation", "legalSourceSha256", "reviewedAt", "reviewer"]))
    throw new Error("Core media license review approval is absent or invalid");
  validateReviewedSource(approval, source, target);
  validateReviewMetadata(approval);
}

function validateReviewedSource(approval, source, target) {
  if (approval.legalSourceSha256 !== mediaLegalSourceIdentity(source, target))
    throw new Error("Core media license review covers different legal or source inputs");
}

function validateReviewMetadata(approval) {
  if (typeof approval.reviewer !== "string" || !approval.reviewer.trim())
    throw new Error("Core media license review requires an accountable reviewer");
  const reviewedAt = Date.parse(approval.reviewedAt);
  if (!Number.isFinite(reviewedAt) || reviewedAt > Date.now())
    throw new Error("Core media license review requires a completed review time");
  if (!validEvidenceLocation(approval.evidenceLocation))
    throw new Error("Core media license review requires a durable HTTPS evidence location");
}

function validEvidenceLocation(location) {
  let evidence;
  try {
    evidence = new URL(location);
  } catch {
    return false;
  }
  return !(
    evidence.protocol !== "https:" ||
    evidence.username ||
    evidence.password ||
    evidence.hash ||
    !evidence.hostname
  );
}

function exactKeys(value, keys) {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    Object.keys(value).toSorted().join("\0") === keys.toSorted().join("\0")
  );
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [repository, target, option] = process.argv.slice(2);
  if (!repository || !target || (option && option !== "--allow-pending"))
    throw new Error("Usage: media-license-policy.mjs <repository> <target> [--allow-pending]");
  const receipt = validateMediaLicenseReview(repository, target, {
    requireApproved: option !== "--allow-pending",
  });
  process.stdout.write(`${JSON.stringify(receipt)}\n`);
}
