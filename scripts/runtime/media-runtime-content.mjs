import { lstatSync, readFileSync } from "node:fs";

import { safeRelativePath } from "./files.mjs";

export class MediaRuntimeContentError extends Error {}

export function parseMediaManifest(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    if (!(error instanceof SyntaxError)) throw error;
    throw new MediaRuntimeContentError("Invalid media manifest", { cause: error });
  }
}

export function runtimeFileMetadata(path, message) {
  try {
    return lstatSync(path);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    throw new MediaRuntimeContentError(message, { cause: error });
  }
}

export function validateMediaProducer(producer, target, version) {
  const expected = { schemaVersion: 1, target, version, relocationVerified: true };
  if (
    Object.entries(expected).some(([key, value]) => producer?.[key] !== value) ||
    !/^[a-f0-9]{40}$/u.test(producer.appCommit ?? "") ||
    !Array.isArray(producer.nativeSupplier?.packages) ||
    producer.nativeSupplier.packages.some((item) => item === null || typeof item !== "object")
  )
    throw new MediaRuntimeContentError("Media runtime producer identity differs from the controlled build");
}

export function validateMediaNativeInventory(manifest) {
  const native = manifest.producer.nativeDependencyFiles;
  const declared = new Set(manifest.files.map((file) => file.path));
  if (
    !Array.isArray(native) ||
    native.length !== manifest.producer.signing.compiledBinaryCount ||
    new Set(native).size !== native.length ||
    !native.includes(manifest.executable) ||
    native.some((path) => !safeRelativePath(path) || !declared.has(path))
  )
    throw new MediaRuntimeContentError("Media native dependency inventory differs from its manifest");
}
