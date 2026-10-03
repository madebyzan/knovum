import { createHash } from "node:crypto";
import { closeSync, lstatSync, openSync, readFileSync, readdirSync, readSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";

const maxManifestBytes = 16 * 1024 * 1024;
const maxRuntimeFileBytes = 4 * 1024 * 1024 * 1024;
const maxRuntimeBytes = 16 * 1024 * 1024 * 1024;
const maxRuntimeFiles = 100_000;
const manifestKeys = [
  "schemaVersion",
  "runtimeId",
  "version",
  "protocolVersion",
  "dependencyLockSha256",
  "target",
  "executable",
  "packageRoot",
  "files",
];

export function validatePythonRuntime(directory, runtimeTarget, dependencyLock) {
  const manifest = readManifest(directory, runtimeTarget);
  const executable = runtimeTarget.startsWith("windows-") ? "python.exe" : "bin/python3";
  validateManifestHeader(manifest, runtimeTarget, executable, dependencyLock);
  const declared = validateManifestFiles(manifest.files, directory, runtimeTarget, executable);
  assertRequiredFiles(declared, executable);
  const actual = listRuntimeFiles(directory, { excludeManifest: true });
  if (actual.length !== declared.size || actual.some((path) => !declared.has(path))) {
    throw new Error(`Python runtime contains files not declared by its manifest for ${runtimeTarget}`);
  }
}

export function listRuntimeFiles(root, options = {}) {
  return walkRuntimeFiles(root, root)
    .filter((path) => !options.excludeManifest || path !== "runtime-manifest.json")
    .toSorted();
}

export function safeRelativePath(path) {
  if (typeof path !== "string" || path.length === 0 || path.trim() !== path) return false;
  if (path.includes("\\") || isAbsolute(path) || /^[A-Za-z]:/u.test(path)) return false;
  return path.split("/").every((part) => part && part !== "." && part !== "..");
}

function sha256File(path) {
  const descriptor = openSync(path, "r");
  const hash = createHash("sha256");
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  try {
    for (let size = readSync(descriptor, buffer); size > 0; size = readSync(descriptor, buffer)) {
      hash.update(buffer.subarray(0, size));
    }
  } finally {
    closeSync(descriptor);
  }
  return hash.digest("hex");
}

function readManifest(directory, runtimeTarget) {
  const path = join(directory, "runtime-manifest.json");
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > maxManifestBytes) {
    throw new Error(`Python runtime manifest is not a bounded regular file for ${runtimeTarget}`);
  }
  return JSON.parse(readFileSync(path, "utf8"));
}

function validateManifestHeader(manifest, target, executable, dependencyLock) {
  if (!validManifestShape(manifest, target, executable)) {
    throw new Error(`Python runtime manifest is invalid for ${target}`);
  }
  if (manifest.dependencyLockSha256 !== sha256File(dependencyLock)) {
    throw new Error(`Python runtime manifest is invalid for ${target}`);
  }
}

function validManifestShape(manifest, target, executable) {
  if (!exactKeys(manifest, manifestKeys) || manifest.schemaVersion !== 1) return false;
  if (manifest.runtimeId !== "cpython" || manifest.version !== "3.11.13") return false;
  if (manifest.protocolVersion !== 1 || manifest.packageRoot !== "packages") return false;
  if (manifest.target !== target || manifest.executable !== executable) return false;
  return Array.isArray(manifest.files) && manifest.files.length <= maxRuntimeFiles;
}

function validateManifestFiles(files, directory, target, executable) {
  const declared = new Set();
  let totalSize = 0;
  for (const file of files) {
    validateManifestEntry(file, declared, target);
    totalSize += file.size;
    if (!Number.isSafeInteger(totalSize) || totalSize > maxRuntimeBytes) {
      throw new Error(`Python runtime size exceeds limit for ${target}`);
    }
    declared.add(file.path);
    validateRuntimeFile(directory, file, target, executable);
  }
  return declared;
}

function validateManifestEntry(file, declared, target) {
  if (!validManifestEntry(file) || declared.has(file.path)) {
    throw new Error(`Python runtime manifest contains an invalid file entry for ${target}`);
  }
}

function validManifestEntry(file) {
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
  for (const required of [executable, "packages/knovum_inference/__main__.py"]) {
    if (!declared.has(required)) throw new Error(`Python runtime manifest does not declare ${required}`);
  }
}

function walkRuntimeFiles(root, directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    const metadata = lstatSync(path);
    assertRegularRuntimeEntry(path, metadata);
    if (metadata.isDirectory()) files.push(...walkRuntimeFiles(root, path));
    else files.push(relative(root, path).replaceAll("\\", "/"));
  }
  return files;
}

function assertRegularRuntimeEntry(path, metadata) {
  if (metadata.isSymbolicLink()) throw new Error(`Runtime contains a link: ${path}`);
  if (!metadata.isFile() && !metadata.isDirectory()) {
    throw new Error(`Runtime contains a non-file: ${path}`);
  }
}

function exactKeys(value, expected) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  return Object.keys(value).toSorted().join("\0") === expected.toSorted().join("\0");
}
