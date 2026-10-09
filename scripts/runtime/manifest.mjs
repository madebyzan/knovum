import { lstatSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import { runtimeProfiles } from "./profiles.mjs";
import { validateRuntimeProvenance } from "./provenance.mjs";

const maxManifestBytes = 16 * 1024 * 1024;
const maxRuntimeFileBytes = 4 * 1024 * 1024 * 1024;
const maxRuntimeBytes = 16 * 1024 * 1024 * 1024;
const maxRuntimeFiles = 100_000;
const manifestKeys = [
  "schemaVersion",
  "runtimeId",
  "version",
  "dependencyRequirementsSha256",
  "profile",
  "dependencyLockSha256",
  "target",
  "executable",
  "packageRoot",
  "files",
];

export function validatePythonRuntime(directory, runtimeTarget, dependencyLock) {
  const manifest = readManifest(directory, runtimeTarget);
  const executable = runtimeTarget.startsWith("windows-") ? "python.exe" : "bin/python3";
  validateManifestHeader(manifest, runtimeTarget, executable, directory);
  const declared = validateManifestFiles(manifest.files, directory, runtimeTarget, executable);
  assertRequiredFiles(declared, executable);
  const actual = listRuntimeFiles(directory, { excludeManifest: true });
  if (actual.length !== declared.size || actual.some((path) => !declared.has(path))) {
    throw new Error(`Python runtime contains files not declared by its manifest for ${runtimeTarget}`);
  }
  if (actual.some((path) => path.startsWith("packages/knovum_") || path.startsWith("build-evidence/"))) {
    throw new Error("Third-party runtime must exclude application Worker and external build evidence");
  }
  validateRuntimeProvenance(directory, manifest, dependencyLock);
}

function readManifest(directory, runtimeTarget) {
  const path = join(directory, "runtime-manifest.json");
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > maxManifestBytes) {
    throw new Error(`Python runtime manifest is not a bounded regular file for ${runtimeTarget}`);
  }
  return JSON.parse(readFileSync(path, "utf8"));
}

function validateManifestHeader(manifest, target, executable, directory) {
  if (!validManifestShape(manifest, target, executable)) {
    throw new Error(`Python runtime manifest is invalid for ${target}`);
  }
  if (manifest.dependencyRequirementsSha256 !== sha256File(join(directory, "runtime-requirements.txt"))) {
    throw new Error(`Python runtime manifest is invalid for ${target}`);
  }
}

function validManifestShape(manifest, target, executable) {
  if (!exactKeys(manifest, manifestKeys)) return false;
  if (manifest.schemaVersion !== 1) return false;
  if (manifest.runtimeId !== "cpython" || manifest.version !== "3.11.17") return false;
  if (!runtimeProfiles.includes(manifest.profile) || !/^[a-f0-9]{64}$/u.test(manifest.dependencyLockSha256))
    return false;
  if (manifest.target !== target || manifest.executable !== executable) return false;
  if (manifest.packageRoot !== "packages") return false;
  return Array.isArray(manifest.files);
}

function validateManifestFiles(files, directory, target, executable) {
  if (files.length > maxRuntimeFiles) throw new Error(`Python runtime contains too many files for ${target}`);
  const declared = new Set();
  let totalSize = 0;
  for (const file of files) {
    validateManifestFile(file, declared, target);
    totalSize += file.size;
    if (!Number.isSafeInteger(totalSize) || totalSize > maxRuntimeBytes) {
      throw new Error(`Python runtime size exceeds limit for ${target}`);
    }
    declared.add(file.path);
    validateRuntimeFile(directory, file, target, executable);
  }
  return declared;
}

function validateManifestFile(file, declared, target) {
  if (!validManifestFile(file) || declared.has(file.path)) {
    throw new Error(`Python runtime manifest contains an invalid file entry for ${target}`);
  }
}

function validManifestFile(file) {
  if (!file || !exactKeys(file, ["path", "size", "sha256"])) return false;
  if (!safeRelativePath(file.path) || file.path === "runtime-manifest.json") return false;
  if (!Number.isSafeInteger(file.size) || file.size < 0 || file.size > maxRuntimeFileBytes) return false;
  return /^[a-f0-9]{64}$/u.test(file.sha256 ?? "");
}

function validateRuntimeFile(directory, file, target, executable) {
  const path = join(directory, ...file.path.split("/"));
  const metadata = lstatSync(path);
  const executableBitMissing =
    !target.startsWith("windows-") && file.path === executable && !(metadata.mode & 0o111);
  if (!metadata.isFile() || metadata.size !== file.size || executableBitMissing) {
    throw new Error(`Python runtime file does not match its manifest: ${file.path}`);
  }
  if (sha256File(path) !== file.sha256) {
    throw new Error(`Python runtime file does not match its manifest: ${file.path}`);
  }
}

function assertRequiredFiles(declared, executable) {
  for (const required of [executable, "runtime-requirements.txt"]) {
    if (!declared.has(required)) throw new Error(`Python runtime manifest does not declare ${required}`);
  }
}

function exactKeys(value, expected) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  return Object.keys(value).toSorted().join("\0") === expected.toSorted().join("\0");
}
