import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

import { listRuntimeFiles, safeRelativePath, sha256File } from "./files.mjs";
import { buildNativeAudio } from "./native-audio.mjs";
import { materializeNativeAliases, relocateNativeLibraries } from "./native-closure.mjs";
import { prepareOnnxRuntime, verifyNativeRuntime as verifyOnnx } from "./native-ort.mjs";
import { buildNativeSpeech } from "./native-speech.mjs";
import { buildNativeWhisper } from "./native-whisper.mjs";
import { hostRuntimeTarget } from "./profiles.mjs";

export { verifyNativeRuntime } from "./native-ort.mjs";
const recipeFiles = [
  "native-source.json",
  "native-audio-source.json",
  "native-sherpa-source.json",
  "native-whisper-source.json",
  "native.mjs",
  "native-ort.mjs",
  "native-audio.mjs",
  "native-speech.mjs",
  "native-whisper.mjs",
  "native-source-zip.mjs",
  "selected-zip.mjs",
  "native-closure.mjs",
  "voice_bridge.cmake",
  "voice_bridge.cpp",
];
export function nativeDirectory(repository, target) {
  const hash = createHash("sha256");
  for (const file of recipeFiles) hash.update(readFileSync(join(repository, "scripts/runtime", file)));
  return join(repository, ".runtime-development/native", target, hash.digest("hex"));
}

export async function prepareNativeRuntime(repository, target = hostRuntimeTarget()) {
  const directory = nativeDirectory(repository, target),
    manifestPath = join(directory, "native-runtime-manifest.json");
  if (existsSync(manifestPath)) {
    verifyPreparedRuntime(directory, target);
    return manifestPath;
  }
  mkdirSync(dirname(directory), { recursive: true });
  const staging = mkdtempSync(join(dirname(directory), ".native-"));
  const scratch = mkdtempSync(join(dirname(directory), ".build-"));
  try {
    const onnxPath = await prepareOnnxRuntime(repository, target);
    cpSync(dirname(onnxPath), staging, { recursive: true });
    rmSync(join(staging, "native-runtime-manifest.json"));
    const configuration = (name) =>
      JSON.parse(readFileSync(join(repository, "scripts/runtime", name), "utf8"));
    await buildNativeAudio(repository, scratch, staging, {
      target,
      configuration: configuration("native-audio-source.json"),
    });
    await buildNativeSpeech(scratch, staging, target, configuration("native-sherpa-source.json"));
    await buildNativeWhisper(scratch, staging, target, configuration("native-whisper-source.json"));
    materializeNativeAliases(staging);
    if (target.startsWith("macos-")) relocateNativeLibraries(staging, listRuntimeFiles(staging));
    const files = listRuntimeFiles(staging).map((path) => ({
      path,
      size: statSync(join(staging, path)).size,
      sha256: sha256File(join(staging, path)),
    }));
    const original = JSON.parse(readFileSync(onnxPath, "utf8"));
    // ONNX Runtime is never mutated; its supplier hash remains independently pinned.
    const manifest = { schemaVersion: 1, target, onnxRuntime: original.onnxRuntime, files };
    writeFileSync(join(staging, "native-runtime-manifest.json"), JSON.stringify(manifest));
    verifyPreparedRuntime(staging, target);
    renameSync(staging, directory);
    return manifestPath;
  } finally {
    rmSync(staging, { recursive: true, force: true });
    rmSync(scratch, { recursive: true, force: true });
  }
}

export function verifyPreparedRuntime(directory, target) {
  const path = join(directory, "native-runtime-manifest.json");
  if (!lstatSync(path).isFile() || statSync(path).size > 1024 ** 2)
    throw new Error("Native manifest must be a bounded regular file");
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  const source = JSON.parse(readFileSync(new URL("./native-source.json", import.meta.url), "utf8"));
  if (
    manifest.schemaVersion !== 1 ||
    manifest.target !== target ||
    !Array.isArray(manifest.files) ||
    manifest.files.length > 256
  )
    throw new Error("Native runtime identity differs");
  const pinned = source.files[target];
  if (!pinned) throw new Error("Unsupported native target");
  verifyOnnx(
    directory,
    { ...manifest, files: manifest.files.filter((file) => pinned.some((item) => item.path === file.path)) },
    source,
    target,
  );
  const names = new Set();
  for (const file of manifest.files) {
    validateEntry(file, names);
    names.add(file.path);
    verifyFile(directory, file);
  }
  verifyCompleteInventory(directory, target, names);
  return manifest;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({ options: { target: { type: "string" } } });
  console.log(
    await prepareNativeRuntime(resolve(dirname(fileURLToPath(import.meta.url)), "../.."), values.target),
  );
}

function validateEntry(file, names) {
  if (
    !safeRelativePath(file.path) ||
    names.has(file.path) ||
    file.size <= 0 ||
    file.size > 2 * 1024 ** 3 ||
    !/^[a-f0-9]{64}$/u.test(file.sha256)
  )
    throw new Error("Invalid native runtime inventory");
}

function verifyCompleteInventory(directory, target, names) {
  const actual = listRuntimeFiles(directory).filter((name) => name !== "native-runtime-manifest.json");
  if (actual.length !== names.size || actual.some((name) => !names.has(name)))
    throw new Error("Native runtime contains unsealed files");
  const extension = target === "windows-x64" ? "dll" : "dylib";
  for (const name of ["audiocpp", "knovum_voice", "sherpa-onnx-c-api"])
    if (!names.has(`compiled-libraries/${name}.${extension}`))
      throw new Error("Native speech inventory is incomplete");
  if (!names.has(`compiled-libraries/whisper/${target === "windows-x64" ? "" : "lib"}whisper.${extension}`))
    throw new Error("Whisper inventory is incomplete");
}

function verifyFile(directory, file) {
  const metadata = lstatSync(join(directory, file.path));
  if (
    !metadata.isFile() ||
    metadata.size !== file.size ||
    sha256File(join(directory, file.path)) !== file.sha256
  )
    throw new Error("Native runtime integrity differs");
}
