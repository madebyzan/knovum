import {
  compareVersions,
  decodeSigningText,
  httpsUrl,
  releaseTargets,
  stableVersion,
} from "./release-policy.mjs";

const platforms = Object.values(releaseTargets).toSorted();

export function validateManifest(manifest) {
  if (!manifest || typeof manifest !== "object") throw new Error("Update manifest must be an object");
  stableVersion(manifest.version);
  validateNotes(manifest.notes);
  validateDate(manifest.pub_date);
  if (
    Object.keys(manifest.platforms ?? {})
      .toSorted()
      .join(",") !== platforms.join(",")
  ) {
    throw new Error("The update must contain exactly the supported desktop targets");
  }
  for (const [target, artifact] of Object.entries(manifest.platforms)) {
    httpsUrl(artifact.url, `${target} download URL`);
    decodeSigningText(artifact.signature, `${target} signature`);
  }
  return manifest;
}

export function createManifest({ version, notes, pubDate, artifacts, baseUrl }) {
  const base = httpsUrl(baseUrl, "Artifact base URL");
  if (!base.pathname.endsWith("/")) base.pathname += "/";
  const entries = {};
  for (const artifact of artifacts) {
    if (!platforms.includes(artifact.target)) throw new Error("Unsupported updater target");
    if (artifact.version !== version || entries[artifact.target]) {
      throw new Error("Mixed versions or duplicate targets in release artifacts");
    }
    if (typeof artifact.file !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(artifact.file)) {
      throw new Error("Artifact file must be a single safe filename");
    }
    entries[artifact.target] = {
      signature: artifact.signature,
      url: new URL(artifact.file, base).href,
    };
  }
  return validateManifest({ version, notes, pub_date: pubDate, platforms: entries });
}

export function assertPromotion(previous, next) {
  validateManifest(next);
  if (!previous) return;
  validateManifest(previous);
  const comparison = compareVersions(next.version, previous.version);
  if (comparison < 0) throw new Error("Refusing to downgrade the stable channel; publish a forward fix");
  if (comparison > 0) return;
  for (const target of platforms) {
    if (next.platforms[target].signature !== previous.platforms[target].signature) {
      throw new Error("An existing version is immutable; rehosting must preserve every signed artifact");
    }
  }
  if (next.notes !== previous.notes || next.pub_date !== previous.pub_date) {
    throw new Error("Rehosting may change download URLs only");
  }
}

function validateNotes(notes) {
  if (typeof notes !== "string" || !notes.trim() || notes.length > 65_536) {
    throw new Error("Reviewed release notes are required");
  }
}

function validateDate(value) {
  if (typeof value !== "string" || !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{1,3})?Z$/.test(value)) {
    throw new Error("A UTC RFC 3339 publication date is required");
  }
  const milliseconds = Date.parse(value);
  if (
    !Number.isFinite(milliseconds) ||
    new Date(milliseconds).toISOString().slice(0, 19) !== value.slice(0, 19)
  ) {
    throw new Error("A valid calendar date is required");
  }
}
