import { createHash } from "node:crypto";
import { lstatSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import { validateMediaLicenseEvidence, validateMediaLicenseReview } from "./media-license-policy.mjs";
import {
  MediaRuntimeContentError,
  parseMediaManifest,
  runtimeFileMetadata,
  validateCompiledBinaryInventory,
  validateMediaProducer,
} from "./media-runtime-content.mjs";
import { recipeBytes, runtimeTargets } from "./profiles.mjs";
import { validateRuntimeSigning } from "./provenance.mjs";

export const mediaManifestName = "media-manifest.json";

export function mediaExecutable(target) {
  if (!Object.values(runtimeTargets).includes(target)) throw new Error(`Unsupported media target: ${target}`);
  return target === "windows-x64" ? "bin/ffmpeg.exe" : "bin/ffmpeg";
}

export function mediaSource(repository, target) {
  const configuration = JSON.parse(
    readFileSync(join(repository, "scripts/runtime/media-source.json"), "utf8"),
  );
  const source = configuration.targets[target];
  if (!source) throw new Error(`Unsupported media target: ${target}`);
  const licenseReview = validateMediaLicenseReview(repository, target, { requireApproved: false });
  return {
    schemaVersion: configuration.schemaVersion,
    version: configuration.version,
    license: configuration.license,
    licenseSource: configuration.licenseSource,
    sourceOffer: configuration.sourceOffer,
    source,
    licenseReviewSha256: licenseReview.sha256,
  };
}

export function mediaRecipeIdentity(repository, target) {
  return createHash("sha256")
    .update(recipeBytes(mediaSource(repository, target)))
    .update(mediaProducerBytes(repository))
    .digest("hex");
}

export function writeMediaManifest(repository, directory, target, producer) {
  const files = listRuntimeFiles(directory)
    .filter((path) => path !== mediaManifestName)
    .map((path) => ({
      path,
      size: lstatSync(join(directory, path)).size,
      sha256: sha256File(join(directory, path)),
    }));
  const manifest = {
    schemaVersion: 1,
    runtimeId: "ffmpeg",
    target,
    version: mediaSource(repository, target).version,
    executable: mediaExecutable(target),
    recipeSha256: mediaRecipeIdentity(repository, target),
    licenseReviewSha256: mediaSource(repository, target).licenseReviewSha256,
    producer,
    files,
  };
  writeFileSync(join(directory, mediaManifestName), `${JSON.stringify(manifest, null, 2)}\n`);
  validateMediaRuntime(repository, directory, target);
}

export function validateMediaRuntime(repository, directory, target) {
  const path = join(directory, mediaManifestName);
  const metadata = runtimeFileMetadata(path, "Invalid media manifest");
  if (!metadata.isFile() || metadata.size > 16 * 1024 ** 2)
    throw new MediaRuntimeContentError("Invalid media manifest");
  const manifest = parseMediaManifest(path);
  const source = mediaSource(repository, target);
  const expected = {
    schemaVersion: 1,
    runtimeId: "ffmpeg",
    target,
    version: source.version,
    executable: mediaExecutable(target),
    recipeSha256: mediaRecipeIdentity(repository, target),
    licenseReviewSha256: source.licenseReviewSha256,
  };
  if (
    !exactKeys(manifest, [...Object.keys(expected), "producer", "files"]) ||
    Object.entries(expected).some(([key, value]) => manifest[key] !== value)
  )
    throw new MediaRuntimeContentError("Media runtime identity differs from the controlled build");
  validateMediaProducer(manifest.producer, target, source.version, source.source);
  const licenseReview = validateMediaLicenseReview(repository, target, { requireApproved: false });
  validateRuntimeSigning(manifest.producer.signing, target, MediaRuntimeContentError);
  validateFiles(directory, manifest, target);
  validateCompiledBinaryInventory(manifest);
  validateMediaLicenseEvidence(
    manifest.producer.licenseReview,
    manifest.files,
    licenseReview,
    MediaRuntimeContentError,
  );
  return manifest;
}

function validateFiles(directory, manifest, target) {
  if (!Array.isArray(manifest.files) || manifest.files.length > 64)
    throw new MediaRuntimeContentError("Invalid media file inventory");
  const declared = new Set();
  let total = 0;
  for (const file of manifest.files) {
    validateManifestFile(file, declared);
    declared.add(file.path);
    total += file.size;
    if (total > 4 * 1024 ** 3) throw new MediaRuntimeContentError("Media runtime exceeds its size limit");
    validateFileContent(directory, file, manifest.executable, target);
  }
  const actual = listRuntimeFiles(directory, { ErrorType: MediaRuntimeContentError }).filter(
    (path) => path !== mediaManifestName,
  );
  if (actual.length !== declared.size || actual.some((path) => !declared.has(path)))
    throw new MediaRuntimeContentError("Media runtime contains undeclared files");
  if (!declared.has(manifest.executable))
    throw new MediaRuntimeContentError("Media runtime has no FFmpeg executable");
}

function exactKeys(value, keys) {
  return (
    value !== null &&
    typeof value === "object" &&
    Object.keys(value).toSorted().join("\0") === keys.toSorted().join("\0")
  );
}

const validSize = (size) => Number.isSafeInteger(size) && size >= 0 && size <= 4 * 1024 ** 3;

function validateManifestFile(file, declared) {
  if (
    !exactKeys(file, ["path", "size", "sha256"]) ||
    !safeRelativePath(file.path) ||
    file.path === mediaManifestName ||
    declared.has(file.path) ||
    !validSize(file.size) ||
    !/^[a-f0-9]{64}$/u.test(file.sha256 ?? "")
  )
    throw new MediaRuntimeContentError("Invalid media file inventory");
}

function validateFileContent(directory, file, executable, target) {
  const path = join(directory, file.path);
  const message = `Media runtime file differs from its manifest: ${file.path}`;
  const metadata = runtimeFileMetadata(path, message);
  try {
    const missingExecutableBit =
      target !== "windows-x64" && file.path === executable && !(metadata.mode & 0o111);
    if (
      !metadata.isFile() ||
      metadata.size !== file.size ||
      missingExecutableBit ||
      sha256File(path) !== file.sha256
    )
      throw new MediaRuntimeContentError(message);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    throw new MediaRuntimeContentError(message, { cause: error });
  }
}

function mediaProducerBytes(repository) {
  const configuration = JSON.parse(
    readFileSync(join(repository, "scripts/runtime/media-source.json"), "utf8"),
  );
  return Buffer.concat(
    configuration.producerSources
      .toSorted()
      .flatMap((name) => [
        Buffer.from(`${name}\0`),
        readFileSync(join(repository, "scripts/runtime", name)),
        Buffer.from("\0"),
      ]),
  );
}
