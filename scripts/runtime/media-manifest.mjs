import { createHash } from "node:crypto";
import { lstatSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import { producerBytes, runtimeTargets } from "./profiles.mjs";
import { validateNativeSupplier, validateSigningRecord } from "./provenance.mjs";

const mediaVersion = "9.0.1";
export const mediaManifestName = "media-manifest.json";

export function mediaExecutable(target) {
  if (!Object.values(runtimeTargets).includes(target)) throw new Error(`Unsupported media target: ${target}`);
  return target === "windows-x64" ? "Library/bin/ffmpeg.exe" : "bin/ffmpeg";
}

export function mediaLockPath(repository, target) {
  return join(repository, "python/native-locks/media", `${target}.json`);
}

function mediaRecipeIdentity(repository, target) {
  return createHash("sha256")
    .update(readFileSync(join(repository, "python/runtime-source.json")))
    .update(producerBytes(repository, "media"))
    .update(readFileSync(mediaLockPath(repository, target)))
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
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > 16 * 1024 ** 2) throw new Error("Invalid media manifest");
  const manifest = JSON.parse(readFileSync(path, "utf8"));
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
    throw new Error("Media runtime identity differs from the controlled build");
  validateProducer(manifest.producer, target);
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  validateNativeSupplier(
    manifest.producer.nativeSupplier,
    configuration.nativeSupplier,
    mediaLockPath(repository, target),
  );
  validateSigningRecord(manifest.producer.signing, target);
  validateFiles(directory, manifest, target);
  validateNativeInventory(manifest);
  return manifest;
}

function validateNativeInventory(manifest) {
  const native = manifest.producer.nativeDependencyFiles;
  const declared = new Set(manifest.files.map((file) => file.path));
  if (
    !Array.isArray(native) ||
    native.length !== manifest.producer.signing.compiledBinaryCount ||
    new Set(native).size !== native.length ||
    !native.includes(manifest.executable) ||
    native.some((path) => !safeRelativePath(path) || !declared.has(path))
  )
    throw new Error("Media native dependency inventory differs from its manifest");
}

function validateFiles(directory, manifest, target) {
  if (!Array.isArray(manifest.files) || manifest.files.length > 100_000)
    throw new Error("Invalid media file inventory");
  const declared = new Set();
  let total = 0;
  for (const file of manifest.files) {
    validateFileEntry(file, declared);
    declared.add(file.path);
    total += file.size;
    if (total > 4 * 1024 ** 3) throw new Error("Media runtime exceeds its size limit");
    validateFileContent(directory, file, manifest.executable, target);
  }
  const actual = listRuntimeFiles(directory).filter((path) => path !== mediaManifestName);
  if (actual.length !== declared.size || actual.some((path) => !declared.has(path)))
    throw new Error("Media runtime contains undeclared files");
  if (!declared.has(manifest.executable)) throw new Error("Media runtime has no FFmpeg executable");
}

function validateProducer(producer, target) {
  const expected = { schemaVersion: 1, target, version: mediaVersion, relocationVerified: true };
  if (
    Object.entries(expected).some(([key, value]) => producer?.[key] !== value) ||
    !/^[a-f0-9]{40}$/u.test(producer.appCommit ?? "")
  )
    throw new Error("Media runtime producer identity differs from the controlled build");
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
    throw new Error("Invalid media file inventory");
}

function validateFileContent(directory, file, executable, target) {
  const path = join(directory, file.path);
  const metadata = lstatSync(path);
  const missingExecutableBit =
    target !== "windows-x64" && file.path === executable && !(metadata.mode & 0o111);
  if (
    !metadata.isFile() ||
    metadata.size !== file.size ||
    missingExecutableBit ||
    sha256File(path) !== file.sha256
  )
    throw new Error(`Media runtime file differs from its manifest: ${file.path}`);
}

export function validateMediaDescriptor(repository, descriptor, target) {
  const expected = {
    schemaVersion: 1,
    runtimeId: "ffmpeg",
    target,
    version: mediaVersion,
    nativeLockSha256: sha256File(mediaLockPath(repository, target)),
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
