import { lstatSync, readFileSync } from "node:fs";

import { recipeBytes } from "./profiles.mjs";

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

export function validateMediaProducer(producer, target, version, source) {
  const expected = { schemaVersion: 1, target, version, relocationVerified: true };
  if (
    Object.entries(expected).some(([key, value]) => producer?.[key] !== value) ||
    !/^[a-f0-9]{40}$/u.test(producer.appCommit ?? "") ||
    !producer.source ||
    !recipeBytes(producer.source).equals(recipeBytes(source))
  )
    throw new MediaRuntimeContentError("Media runtime producer identity differs from the controlled build");
}

export function validateCompiledBinaryInventory(manifest) {
  const binaries = manifest.producer.compiledBinaryFiles;
  if (
    !Array.isArray(binaries) ||
    binaries.length !== 1 ||
    binaries[0] !== manifest.executable ||
    manifest.producer.signing.compiledBinaryCount !== 1 ||
    manifest.files.some(
      ({ path }) =>
        path !== manifest.executable &&
        ((!path.startsWith("notices/") && !path.startsWith("build-evidence/")) ||
          /\.(?:exe|dll|dylib|so)(?:\.|$)/u.test(path)),
    )
  )
    throw new MediaRuntimeContentError("Media compiled binary inventory must contain only FFmpeg");
}
