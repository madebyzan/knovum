import { execFileSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  lstatSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";

import { compiledFiles, signCompiledFiles, verifyCompiledSignature } from "./compiled-signing.mjs";
import { activateDirectory, listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import { prepareNativeRuntime } from "./native.mjs";
import { hostRuntimeTarget } from "./profiles.mjs";
import { requireWindowsNativeClosure } from "./windows-native-closure.mjs";
import { installWindowsNativeDependencies } from "./windows-vc-runtime.mjs";

export async function prepareNativePackage(repository, { production = false } = {}) {
  const target = hostRuntimeTarget();
  const supplierManifest = await prepareNativeRuntime(repository, target);
  execFileSync(
    "cargo",
    [
      "build",
      "--locked",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "-p",
      "knovum-inference-worker",
      "--release",
    ],
    { cwd: repository, stdio: "inherit" },
  );
  const destination = join(repository, "src-tauri/resources/native-runtime");
  mkdirSync(dirname(destination), { recursive: true });
  const staging = mkdtempSync(join(dirname(destination), ".native-package-"));
  try {
    cpSync(dirname(supplierManifest), staging, { recursive: true });
    rmSync(join(staging, "native-runtime-manifest.json"));
    const name = target === "windows-x64" ? "knovum-inference-worker.exe" : "knovum-inference-worker";
    copyFileSync(join(repository, "src-tauri/target/release", name), join(staging, "bin", name));
    chmodSync(join(staging, "bin", name), 0o755);
    if (target === "windows-x64") await installWindowsNativeDependencies(staging, repository);
    const signing = signCompiledFiles(
      staging,
      target,
      production ? "production" : target.startsWith("macos-") ? "adhoc" : "unsigned",
      { preserveMicrosoftRuntime: true },
    );
    const evidence = join(staging, "build-evidence");
    mkdirSync(evidence, { recursive: true });
    for (const file of [
      "native-source.json",
      "native-audio-source.json",
      "native-sherpa-source.json",
      "native-whisper-source.json",
    ])
      copyFileSync(join(repository, "scripts/runtime", file), join(evidence, file));
    writeFileSync(join(evidence, "signing.json"), `${JSON.stringify(signing, null, 2)}\n`);
    const original = JSON.parse(readFileSync(supplierManifest, "utf8"));
    const files = listRuntimeFiles(staging).map((path) => ({
      path,
      size: statSync(join(staging, path)).size,
      sha256: sha256File(join(staging, path)),
    }));
    const manifest = { schemaVersion: 1, target, onnxRuntime: original.onnxRuntime, files };
    writeFileSync(join(staging, "native-runtime-manifest.json"), JSON.stringify(manifest));
    validateNativePackage(staging, target, manifest, production);
    if (target === "windows-x64") requireWindowsNativeClosure(staging, repository);
    activateDirectory(staging, destination);
    return manifest;
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

// The caller supplies the build-trusted manifest; a packaged manifest cannot authorize itself.
export function validateNativePackage(directory, target, trusted, production = false) {
  const manifest = readManifest(directory);
  validateManifestIdentity(manifest, target, trusted);
  const names = new Set();
  for (const file of manifest.files) {
    validateEntry(file, names);
    names.add(file.path);
    const path = join(directory, file.path);
    const metadata = lstatSync(path);
    if (!metadata.isFile() || metadata.size !== file.size || sha256File(path) !== file.sha256)
      throw new Error("Packaged native bytes differ");
  }
  const actual = listRuntimeFiles(directory).filter((p) => p !== "native-runtime-manifest.json");
  if (!completeInventory(actual, names)) throw new Error("Packaged native runtime contains unverified files");
  validateExecutables(directory, target, { manifest, names, production });
  return manifest;
}

function validateEntry(file, names) {
  if (
    !safeRelativePath(file.path) ||
    names.has(file.path) ||
    !Number.isSafeInteger(file.size) ||
    file.size < 1 ||
    file.size > 2 * 1024 ** 3 ||
    !/^[a-f0-9]{64}$/u.test(file.sha256)
  )
    throw new Error("Invalid native inventory entry");
}

function completeInventory(actual, names) {
  return actual.length === names.size && actual.every((path) => names.has(path));
}

function validateManifestIdentity(manifest, target, trusted) {
  if (
    manifest.schemaVersion !== 1 ||
    manifest.target !== target ||
    JSON.stringify(manifest) !== JSON.stringify(trusted)
  )
    throw new Error("Packaged native identity differs from the controlled build");
  if (!Array.isArray(manifest.files) || !manifest.files.length || manifest.files.length > 256)
    throw new Error("Invalid native inventory");
}

function validateExecutables(directory, target, { manifest, names, production }) {
  const worker = `bin/knovum-inference-worker${target === "windows-x64" ? ".exe" : ""}`;
  if (!names.has(worker) || !names.has(manifest.onnxRuntime))
    throw new Error("Native package lacks its worker or ONNX Runtime");
  if (production)
    for (const path of compiledFiles(directory, target))
      verifyCompiledSignature(path, target, true, { preserveMicrosoftRuntime: true });
}

function readManifest(directory) {
  const root = lstatSync(directory);
  if (!root.isDirectory() || root.isSymbolicLink())
    throw new Error("Native package requires a regular directory");
  const path = join(directory, "native-runtime-manifest.json"),
    metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > 1024 ** 2)
    throw new Error("Native manifest must be a bounded regular file");
  return JSON.parse(readFileSync(path, "utf8"));
}
