import { createHash, randomUUID } from "node:crypto";
import {
  closeSync,
  lstatSync,
  openSync,
  readSync,
  readdirSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
if (args.length !== 4 || args[0] !== "--target" || args[2] !== "--directory") {
  throw new Error("Usage: pnpm runtime:manifest --target <target> --directory <signed-python-runtime>");
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
for (const required of [executable, "packages/knovum_inference/__main__.py"]) {
  if (!files.some((file) => file.path === required)) {
    throw new Error(`Python runtime is missing ${required}`);
  }
}
if (!target.startsWith("windows-")) {
  const python = lstatSync(join(root, executable));
  if (!(python.mode & 0o111)) throw new Error("Bundled Python is not executable");
}

const manifest = {
  schemaVersion: 1,
  runtimeId: "cpython",
  version: "3.11.13",
  protocolVersion: 1,
  dependencyLockSha256: sha256File(join(repository, "python", "uv.lock")),
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
  visit(runtimeRoot, runtimeRoot, state);
  return state.files.toSorted((left, right) => left.path.localeCompare(right.path));
}

function visit(runtimeRoot, directory, state) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    const metadata = lstatSync(path);
    assertRegularEntry(path, metadata);
    if (metadata.isDirectory()) visit(runtimeRoot, path, state);
    else addFile(runtimeRoot, path, metadata, state);
  }
}

function assertRegularEntry(path, metadata) {
  if (metadata.isSymbolicLink()) throw new Error(`Runtime contains a link: ${path}`);
  if (!metadata.isFile() && !metadata.isDirectory()) {
    throw new Error(`Runtime contains a non-file: ${path}`);
  }
}

function addFile(runtimeRoot, path, metadata, state) {
  const name = relative(runtimeRoot, path).replaceAll("\\", "/");
  if (name === "runtime-manifest.json") return;
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
  if (name.split("/").some((part) => !part || part === "." || part === "..")) {
    throw new Error(`Runtime contains an unsafe path: ${name}`);
  }
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
