import { createHash } from "node:crypto";
import {
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
import { dirname, join } from "node:path";

import { listRuntimeFiles, safeRelativePath, sha256File } from "../files.mjs";
import { materializeLibraryAliases } from "./libraries.mjs";

export function recipeIdentity(repository, sources) {
  const hash = createHash("sha256");
  for (const file of sources)
    hash
      .update(file)
      .update("\0")
      .update(readFileSync(join(repository, "scripts/runtime", file)));
  hash.update(readFileSync(join(repository, "src-tauri/tauri.conf.json")));
  return hash.digest("hex");
}

export async function prepareSupplier(repository, target, { name, sources, build }) {
  const identity = recipeIdentity(repository, [
    ...sources.map((file) => `inference/${file}`),
    "transfer.mjs",
    "files.mjs",
    "inference/libraries.mjs",
  ]);
  const directory = join(repository, ".runtime-development/inference-suppliers", target, name, identity);
  if (existsSync(directory)) {
    verifySupplier(directory, target, name);
    console.log(`Reusing verified inference supplier: ${name}`);
    return directory;
  }
  mkdirSync(dirname(directory), { recursive: true });
  const output = mkdtempSync(join(dirname(directory), ".install-"));
  const scratch = mkdtempSync(join(dirname(directory), ".build-"));
  try {
    console.log(`Preparing inference supplier: ${name}`);
    mkdirSync(join(output, "compiled-libraries"));
    await build(scratch, output);
    materializeLibraryAliases(output);
    const files = listRuntimeFiles(output).map((path) => ({
      path,
      size: statSync(join(output, path)).size,
      sha256: sha256File(join(output, path)),
    }));
    writeFileSync(
      join(output, "supplier-manifest.json"),
      JSON.stringify({ schemaVersion: 1, target, name, files }),
    );
    verifySupplier(output, target, name);
    renameSync(output, directory);
    return directory;
  } finally {
    rmSync(output, { recursive: true, force: true });
    rmSync(scratch, { recursive: true, force: true });
  }
}

function verifySupplier(directory, target, name) {
  const path = join(directory, "supplier-manifest.json");
  if (!lstatSync(path).isFile() || statSync(path).size > 1024 ** 2)
    throw new Error("Inference supplier manifest must be a bounded regular file");
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  if (manifest.schemaVersion !== 1 || manifest.target !== target || manifest.name !== name)
    throw new Error("Inference supplier identity differs");
  verifyInventory(directory, manifest.files, "supplier-manifest.json");
}

export function verifyInventory(directory, files, manifestName) {
  if (!Array.isArray(files) || !files.length || files.length > 256)
    throw new Error("Invalid inference runtime inventory");
  const names = new Set();
  for (const file of files) {
    validateEntry(file, names);
    names.add(file.path);
    const metadata = lstatSync(join(directory, file.path));
    if (
      !metadata.isFile() ||
      metadata.size !== file.size ||
      sha256File(join(directory, file.path)) !== file.sha256
    )
      throw new Error("Inference runtime integrity differs");
  }
  const actual = listRuntimeFiles(directory).filter((file) => file !== manifestName);
  if (actual.length !== names.size || actual.some((file) => !names.has(file)))
    throw new Error("Inference runtime contains unsealed files");
  return names;
}

function validateEntry(file, names) {
  if (
    !safeRelativePath(file.path) ||
    names.has(file.path) ||
    file.size <= 0 ||
    file.size > 2 * 1024 ** 3 ||
    !/^[a-f0-9]{64}$/u.test(file.sha256)
  )
    throw new Error("Invalid inference runtime inventory");
}
