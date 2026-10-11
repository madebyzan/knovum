import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  lstatSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

import { sha256File } from "./files.mjs";
import { hostRuntimeTarget } from "./profiles.mjs";
import { extractSelectedZip } from "./selected-zip.mjs";
import { downloadArtifact } from "./transfer.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

function nativeDirectory(root, target) {
  const identity = createHash("sha256")
    .update(readFileSync(join(root, "scripts/runtime/native-source.json")))
    .update(readFileSync(join(root, "scripts/runtime/native-ort.mjs")))
    .digest("hex");
  return join(root, ".runtime-development/native-ort", target, identity);
}

export async function prepareOnnxRuntime(root, target = hostRuntimeTarget()) {
  const source = JSON.parse(readFileSync(join(root, "scripts/runtime/native-source.json"), "utf8"));
  if (!source.files[target]) throw new Error(`Unsupported native target: ${target}`);
  const directory = nativeDirectory(root, target);
  const manifestPath = join(directory, "native-runtime-manifest.json");
  if (existsSync(manifestPath)) {
    if (!lstatSync(manifestPath).isFile()) throw new Error("Native manifest must be a regular file");
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    verifyNativeRuntime(directory, manifest, source, target);
    return manifestPath;
  }
  const platform = target.startsWith("macos-") ? "macos" : target === "windows-x64" ? "windows" : undefined;
  if (!platform) throw new Error(`Unsupported native target: ${target}`);
  mkdirSync(dirname(directory), { recursive: true });
  const staging = mkdtempSync(join(dirname(directory), ".native-"));
  try {
    const archive = join(staging, "supplier.whl");
    await downloadArtifact(source.onnxRuntime[platform], archive);
    const files = await extractNative(archive, staging, platform);
    rmSync(archive);
    const onnxRuntime = platform === "macos" ? "libonnxruntime.1.22.1.dylib" : "onnxruntime.dll";
    if (!files.some((file) => file.path === onnxRuntime))
      throw new Error("Supplier lacks the native ONNX Runtime library");
    const manifest = { schemaVersion: 1, target, onnxRuntime, files };
    verifyNativeRuntime(staging, manifest, source, target);
    writeFileSync(join(staging, "native-runtime-manifest.json"), JSON.stringify(manifest));
    renameSync(staging, directory);
    return manifestPath;
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

export function verifyNativeRuntime(directory, manifest, source, target) {
  const expected = source.files[target];
  if (
    manifest.schemaVersion !== 1 ||
    manifest.target !== target ||
    !expected ||
    !Array.isArray(manifest.files) ||
    manifest.files.length !== expected.length ||
    !expected.some((file) => file.path === manifest.onnxRuntime)
  )
    throw new Error("Native runtime inventory differs");
  const names = new Set();
  for (const file of manifest.files) {
    verifyPinnedEntry(file, expected, names);
    names.add(file.path);
    verifyFile(directory, file);
  }
}

async function extractNative(archive, directory, platform) {
  const selected = new Map([
    ["onnxruntime/LICENSE", "ONNX-RUNTIME-LICENSE"],
    ["onnxruntime/ThirdPartyNotices.txt", "ONNX-RUNTIME-NOTICES"],
    [
      platform === "macos"
        ? "onnxruntime/capi/libonnxruntime.1.22.1.dylib"
        : "onnxruntime/capi/onnxruntime.dll",
      platform === "macos" ? "libonnxruntime.1.22.1.dylib" : "onnxruntime.dll",
    ],
    ...(platform === "windows"
      ? [["onnxruntime/capi/onnxruntime_providers_shared.dll", "onnxruntime_providers_shared.dll"]]
      : []),
  ]);
  await extractSelectedZip(
    archive,
    new Map([...selected].map(([entry, name]) => [entry, join(directory, name)])),
  );
  return [...selected.values()]
    .map((path) => ({
      path,
      size: statSync(join(directory, path)).size,
      sha256: sha256File(join(directory, path)),
    }))
    .toSorted((a, b) => a.path.localeCompare(b.path, "en"));
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({ options: { target: { type: "string" } } });
  console.log(await prepareOnnxRuntime(repository, values.target));
}

function verifyPinnedEntry(file, expected, names) {
  const pinned = expected.find((item) => item.path === file.path);
  if (!pinned || names.has(file.path) || file.size !== pinned.size || file.sha256 !== pinned.sha256)
    throw new Error("Native runtime supplier identity differs");
}

function verifyFile(directory, file) {
  const path = join(directory, file.path);
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size !== file.size || sha256File(path) !== file.sha256)
    throw new Error("Native runtime integrity check failed");
}
