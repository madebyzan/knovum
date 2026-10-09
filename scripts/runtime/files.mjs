import { createHash, randomUUID } from "node:crypto";
import {
  closeSync,
  existsSync,
  lstatSync,
  mkdirSync,
  openSync,
  readdirSync,
  readSync,
  renameSync,
  rmSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";

export function activateDirectory(staged, destination) {
  mkdirSync(dirname(destination), { recursive: true });
  const previous = `${destination}.previous-${randomUUID()}`;
  if (existsSync(destination)) renameSync(destination, previous);
  try {
    renameSync(staged, destination);
  } catch (error) {
    if (existsSync(previous)) {
      try {
        renameSync(previous, destination);
      } catch (restoreError) {
        throw new AggregateError(
          [error, restoreError],
          `Resource activation failed; previous resource preserved at ${previous}`,
        );
      }
    }
    throw error;
  }
  rmSync(previous, { recursive: true, force: true });
}

export function listRuntimeFiles(root, options = {}) {
  return walkRuntimeFiles(root, root, options.ErrorType ?? Error)
    .filter((path) => !options.excludeManifest || path !== "runtime-manifest.json")
    .toSorted();
}

export function safeRelativePath(path) {
  if (typeof path !== "string" || path.length === 0 || path.trim() !== path) return false;
  if (path.includes("\\") || isAbsolute(path) || /^[A-Za-z]:/u.test(path)) return false;
  return path.split("/").every((part) => part && part !== "." && part !== "..");
}

export function sha256File(path) {
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

function walkRuntimeFiles(root, directory, ErrorType) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    const metadata = lstatSync(path);
    assertRuntimePathKind(path, metadata, ErrorType);
    if (metadata.isDirectory()) files.push(...walkRuntimeFiles(root, path, ErrorType));
    else files.push(relative(root, path).replaceAll("\\", "/"));
  }
  return files;
}

function assertRuntimePathKind(path, metadata, ErrorType) {
  if (metadata.isSymbolicLink()) throw new ErrorType(`Runtime contains a link: ${path}`);
  if (!metadata.isFile() && !metadata.isDirectory()) {
    throw new ErrorType(`Runtime contains a non-file: ${path}`);
  }
}
