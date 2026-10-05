import { createHash } from "node:crypto";
import { closeSync, lstatSync, openSync, readdirSync, readSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";

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
