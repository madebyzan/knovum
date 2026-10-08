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
    !Array.isArray(producer.condaSupplier?.packages) ||
    producer.condaSupplier.packages.some((item) => item === null || typeof item !== "object")
  )
    throw new MediaRuntimeContentError("Media runtime producer identity differs from the controlled build");
}

export function validateCompiledDependencyInventory(manifest) {
  const dependencies = manifest.producer.compiledDependencyFiles;
  const declared = new Set(manifest.files.map((file) => file.path));
  if (
    !Array.isArray(dependencies) ||
    dependencies.length !== manifest.producer.signing.compiledBinaryCount ||
    new Set(dependencies).size !== dependencies.length ||
    !dependencies.includes(manifest.executable) ||
    dependencies.some((path) => !safeRelativePath(path) || !declared.has(path))
  )
    throw new MediaRuntimeContentError("Media compiled dependency inventory differs from its manifest");
}
