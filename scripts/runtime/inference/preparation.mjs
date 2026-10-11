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
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

import { listRuntimeFiles, sha256File } from "../files.mjs";
import { hostRuntimeTarget } from "../profiles.mjs";
import { buildAudioLibraries } from "./audio.mjs";
import { materializeLibraryAliases, relocateLibraries } from "./libraries.mjs";
import { prepareOnnxRuntime, verifyOnnxRuntime as verifyOnnx } from "./onnx.mjs";
import { buildSpeechLibraries } from "./speech.mjs";
import { prepareSupplier, recipeIdentity, verifyInventory } from "./suppliers.mjs";
import { buildWhisperLibraries } from "./whisper.mjs";

const PREPARATION_SOURCES = [
  "inference/onnx-source.json",
  "inference/audio-source.json",
  "inference/sherpa-source.json",
  "inference/whisper-source.json",
  "inference/preparation.mjs",
  "inference/suppliers.mjs",
  "inference/onnx.mjs",
  "inference/audio.mjs",
  "inference/speech.mjs",
  "inference/whisper.mjs",
  "inference/source-archive.mjs",
  "selected-zip.mjs",
  "inference/libraries.mjs",
  "inference/voice_bridge.cmake",
  "inference/voice_bridge.cpp",
  "transfer.mjs",
  "files.mjs",
];
export function inferenceDirectory(repository, target) {
  return join(
    repository,
    ".runtime-development/inference",
    target,
    recipeIdentity(repository, PREPARATION_SOURCES),
  );
}

export async function prepareInferenceRuntime(repository, target = hostRuntimeTarget()) {
  const directory = inferenceDirectory(repository, target),
    manifestPath = join(directory, "inference-runtime-manifest.json");
  if (existsSync(manifestPath)) {
    verifyPreparedRuntime(directory, target);
    return manifestPath;
  }
  mkdirSync(dirname(directory), { recursive: true });
  const staging = mkdtempSync(join(dirname(directory), ".inference-"));
  try {
    const onnxPath = await prepareOnnxRuntime(repository, target);
    cpSync(dirname(onnxPath), staging, { recursive: true });
    rmSync(join(staging, "inference-runtime-manifest.json"));
    const configuration = (name) =>
      JSON.parse(readFileSync(join(repository, "scripts/runtime", name), "utf8"));
    for (const [name, sources, build] of [
      [
        "audio",
        ["audio-source.json", "audio.mjs", "voice_bridge.cmake", "voice_bridge.cpp"],
        (scratch, output) =>
          buildAudioLibraries(repository, scratch, output, {
            target,
            configuration: configuration("inference/audio-source.json"),
          }),
      ],
      [
        "speech",
        ["sherpa-source.json", "speech.mjs", "source-archive.mjs", "onnx-source.json"],
        (scratch, output) =>
          buildSpeechLibraries(scratch, output, target, {
            configuration: configuration("inference/sherpa-source.json"),
            onnxDirectory: dirname(onnxPath),
          }),
      ],
      [
        "whisper",
        ["whisper-source.json", "whisper.mjs"],
        (scratch, output) =>
          buildWhisperLibraries(scratch, output, target, configuration("inference/whisper-source.json")),
      ],
    ]) {
      // Supplier builds are deliberately sequential to bound memory and CPU use.
      // oxlint-disable-next-line no-await-in-loop
      const supplier = await prepareSupplier(repository, target, { name, sources, build });
      cpSync(supplier, staging, {
        recursive: true,
        filter: (path) => basename(path) !== "supplier-manifest.json",
      });
    }
    materializeLibraryAliases(staging);
    if (target.startsWith("macos-")) relocateLibraries(staging, listRuntimeFiles(staging));
    const files = listRuntimeFiles(staging).map((path) => ({
      path,
      size: statSync(join(staging, path)).size,
      sha256: sha256File(join(staging, path)),
    }));
    const original = JSON.parse(readFileSync(onnxPath, "utf8"));
    // ONNX Runtime is never mutated; its supplier hash remains independently pinned.
    const manifest = { schemaVersion: 1, target, onnxRuntime: original.onnxRuntime, files };
    writeFileSync(join(staging, "inference-runtime-manifest.json"), JSON.stringify(manifest));
    verifyPreparedRuntime(staging, target);
    renameSync(staging, directory);
    return manifestPath;
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

export function verifyPreparedRuntime(directory, target) {
  const path = join(directory, "inference-runtime-manifest.json");
  if (!lstatSync(path).isFile() || statSync(path).size > 1024 ** 2)
    throw new Error("Inference manifest must be a bounded regular file");
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  const source = JSON.parse(readFileSync(new URL("./onnx-source.json", import.meta.url), "utf8"));
  if (
    manifest.schemaVersion !== 1 ||
    manifest.target !== target ||
    !Array.isArray(manifest.files) ||
    manifest.files.length > 256
  )
    throw new Error("Inference runtime identity differs");
  const pinned = source.files[target];
  if (!pinned) throw new Error("Unsupported inference target");
  verifyOnnx(
    directory,
    { ...manifest, files: manifest.files.filter((file) => pinned.some((item) => item.path === file.path)) },
    source,
    target,
  );
  const names = verifyInventory(directory, manifest.files, "inference-runtime-manifest.json");
  verifyCompleteInventory(target, names);
  return manifest;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({ options: { target: { type: "string" } } });
  console.log(
    await prepareInferenceRuntime(
      resolve(dirname(fileURLToPath(import.meta.url)), "../../.."),
      values.target,
    ),
  );
}

function verifyCompleteInventory(target, names) {
  const extension = target === "windows-x64" ? "dll" : "dylib";
  for (const name of ["audiocpp", "knovum_voice", "sherpa-onnx-c-api"])
    if (!names.has(`compiled-libraries/${name}.${extension}`))
      throw new Error("Inference speech inventory is incomplete");
  if (!names.has(`compiled-libraries/whisper/${target === "windows-x64" ? "" : "lib"}whisper.${extension}`))
    throw new Error("Whisper inventory is incomplete");
}
