import { randomUUID } from "node:crypto";
import { lstatSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import { runtimeProject } from "./profiles.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
if (args.length !== 6 || args[0] !== "--target" || args[2] !== "--directory" || args[4] !== "--profile") {
  throw new Error(
    "Usage: node scripts/runtime/write-manifest.mjs --target <target> --directory <signed-python-runtime> --profile <inference|agent>",
  );
}
const target = args[1];
if (!["macos-arm64", "windows-x64"].includes(target)) {
  throw new Error(`Unsupported runtime target: ${target}`);
}
const host = `${{ darwin: "macos", win32: "windows" }[process.platform]}-${process.arch}`;
if (target !== host) throw new Error(`Runtime manifest must be written on its native target (${host})`);
const root = resolve(args[3]);
const rootMetadata = lstatSync(root);
if (!rootMetadata.isDirectory() || rootMetadata.isSymbolicLink()) {
  throw new Error("Python runtime root must be a real directory");
}
const executable = target.startsWith("windows-") ? "python.exe" : "bin/python3";
const files = collectRuntimeFiles(root);
for (const required of [executable, "runtime-requirements.txt"]) {
  if (!files.some((file) => file.path === required)) {
    throw new Error(`Python runtime is missing ${required}`);
  }
}
if (!target.startsWith("windows-")) {
  const python = lstatSync(join(root, executable));
  if (!(python.mode & 0o111)) throw new Error("Runtime Python is not executable");
}

const profile = args[5];
const project = runtimeProject(repository, profile);
const manifest = {
  schemaVersion: 1,
  runtimeId: "cpython",
  version: "3.11.17",
  dependencyRequirementsSha256: sha256File(join(root, "runtime-requirements.txt")),
  profile,
  dependencyLockSha256: sha256File(join(project, "uv.lock")),
  target,
  executable,
  packageRoot: "packages",
  files,
};
const bytes = `${JSON.stringify(manifest, null, 2)}\n`;
if (Buffer.byteLength(bytes) > 16 * 1024 ** 2) throw new Error("Runtime manifest exceeds 16 MiB");
const temporary = join(root, `.runtime-manifest-${randomUUID()}.part`);
try {
  writeFileSync(temporary, bytes, { flag: "wx" });
  renameSync(temporary, join(root, "runtime-manifest.json"));
} finally {
  rmSync(temporary, { force: true });
}
console.log(`Wrote ${target} runtime manifest for ${files.length} files`);

function collectRuntimeFiles(runtimeRoot) {
  const state = { files: [], totalBytes: 0 };
  for (const name of listRuntimeFiles(runtimeRoot, { excludeManifest: true })) {
    const path = join(runtimeRoot, name);
    addFile(name, path, lstatSync(path), state);
  }
  return state.files;
}

function addFile(name, path, metadata, state) {
  validateFileName(name);
  if (metadata.size > 4 * 1024 ** 3) throw new Error(`Runtime file exceeds 4 GiB: ${name}`);
  state.totalBytes += metadata.size;
  if (state.totalBytes > 16 * 1024 ** 3 || state.files.length >= 100_000) {
    throw new Error("Python runtime exceeds its size or file-count limit");
  }
  state.files.push({ path: name, size: metadata.size, sha256: sha256File(path) });
}

function validateFileName(name) {
  if (name.startsWith(".runtime-manifest-") && name.endsWith(".part")) {
    throw new Error(`Runtime contains an incomplete manifest write: ${name}`);
  }
  if (!safeRelativePath(name)) {
    throw new Error(`Runtime contains an unsafe path: ${name}`);
  }
}
