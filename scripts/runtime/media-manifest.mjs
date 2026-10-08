import { createHash } from "node:crypto";
import { lstatSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import {
  MediaRuntimeContentError,
  parseMediaManifest,
  runtimeFileMetadata,
  validateCompiledDependencyInventory,
  validateMediaProducer,
} from "./media-runtime-content.mjs";
import { producerBytes, runtimeTargets } from "./profiles.mjs";
import { validateCondaSupplier, validateSigningRecord } from "./provenance.mjs";

const mediaVersion = "9.0.1";
export const mediaManifestName = "media-manifest.json";

export function mediaExecutable(target) {
  if (!Object.values(runtimeTargets).includes(target)) throw new Error(`Unsupported media target: ${target}`);
  return target === "windows-x64" ? "Library/bin/ffmpeg.exe" : "bin/ffmpeg";
}

export function mediaCondaLockPath(repository, target) {
  return join(repository, "python/conda-locks/media", `${target}.json`);
}

function mediaRecipeIdentity(repository, target) {
  return createHash("sha256")
    .update(readFileSync(join(repository, "python/runtime-source.json")))
    .update(producerBytes(repository, "media"))
    .update(readFileSync(mediaCondaLockPath(repository, target)))
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
    version: mediaVersion,
    executable: mediaExecutable(target),
    recipeSha256: mediaRecipeIdentity(repository, target),
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
  const expected = {
    schemaVersion: 1,
    runtimeId: "ffmpeg",
    target,
    version: mediaVersion,
    executable: mediaExecutable(target),
    recipeSha256: mediaRecipeIdentity(repository, target),
  };
  if (
    !exactKeys(manifest, [...Object.keys(expected), "producer", "files"]) ||
    Object.entries(expected).some(([key, value]) => manifest[key] !== value)
  )
    throw new MediaRuntimeContentError("Media runtime identity differs from the controlled build");
  validateMediaProducer(manifest.producer, target, mediaVersion);
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  validateCondaSupplier(
    manifest.producer.condaSupplier,
    configuration.condaSupplier,
    mediaCondaLockPath(repository, target),
    MediaRuntimeContentError,
  );
  validateSigningRecord(manifest.producer.signing, target, MediaRuntimeContentError);
  validateFiles(directory, manifest, target);
  validateCompiledDependencyInventory(manifest);
  return manifest;
}

function validateFiles(directory, manifest, target) {
  if (!Array.isArray(manifest.files) || manifest.files.length > 100_000)
    throw new MediaRuntimeContentError("Invalid media file inventory");
  const declared = new Set();
  let total = 0;
  for (const file of manifest.files) {
    validateFileEntry(file, declared);
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

function validSize(size) {
  return Number.isSafeInteger(size) && size >= 0 && size <= 4 * 1024 ** 3;
}

function validateFileEntry(file, declared) {
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

export function validateMediaDescriptor(repository, descriptor, target) {
  const expected = {
    schemaVersion: 1,
    runtimeId: "ffmpeg",
    target,
    version: mediaVersion,
    condaLockSha256: sha256File(mediaCondaLockPath(repository, target)),
  };
  if (
    !exactKeys(descriptor, [...Object.keys(expected), "archiveUrl", "size", "sha256", "manifestSha256"]) ||
    Object.entries(expected).some(([key, value]) => descriptor[key] !== value) ||
    !validSize(descriptor.size) ||
    descriptor.size === 0 ||
    [descriptor.sha256, descriptor.manifestSha256].some((hash) => !/^[a-f0-9]{64}$/u.test(hash ?? ""))
  )
    throw new Error("Invalid core media publication descriptor");
  const url = new URL(descriptor.archiveUrl);
  if (url.protocol !== "https:" || url.username || url.password || url.hash)
    throw new Error("Invalid core media archive URL");
}
