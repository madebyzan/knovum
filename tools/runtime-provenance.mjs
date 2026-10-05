import { lstatSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { listRuntimeFiles, sha256File } from "./runtime-files.mjs";

export function validateRuntimeProvenance(directory, manifest, dependencyLock) {
  const configuration = JSON.parse(
    readFileSync(join(dirname(dependencyLock), "runtime-source.json"), "utf8"),
  );
  const path = join(directory, "build-evidence", "producer.json");
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > 16 * 1024 * 1024)
    throw new Error("Runtime producer record is invalid");
  const record = JSON.parse(readFileSync(path, "utf8"));
  validateProducerIdentity(record, manifest, configuration);
  validateEvidenceFiles(manifest);
  validateCompiledSources(manifest, dependencyLock);
  validateVoxCpm(record.voxcpm, manifest, dependencyLock);
  validateWorkerSource(manifest, dependencyLock);
}

function validateProducerIdentity(record, manifest, configuration) {
  const expected = {
    schemaVersion: 1,
    target: manifest.target,
    version: manifest.version,
    uvVersion: configuration.uvVersion,
    standaloneRelease: configuration.standaloneRelease,
    dependencyLockSha256: manifest.dependencyLockSha256,
    relocationVerified: true,
  };
  if (
    Object.entries(expected).some(([key, value]) => record[key] !== value) ||
    !/^[a-f0-9]{40}$/u.test(record.appCommit ?? "")
  )
    throw new Error("Runtime producer identity differs from the controlled build");
  const source = configuration.targets[manifest.target];
  if (["url", "sha256", "size"].some((key) => record.interpreterSource?.[key] !== source[key]))
    throw new Error("Runtime producer identity differs from the controlled build");
  validateSigningRecord(record.signing, manifest.target);
}

function validateSigningRecord(signing, target) {
  const modes = target === "macos-arm64" ? ["adhoc", "production"] : ["unsigned", "production"];
  if (
    !modes.includes(signing?.mode) ||
    !Number.isSafeInteger(signing?.compiledBinaryCount) ||
    signing.compiledBinaryCount < 1
  )
    throw new Error("Runtime producer signing record is invalid");
}

function validateEvidenceFiles(manifest) {
  for (const name of [
    "producer.json",
    "dependency-evidence.json",
    "sbom.cdx.json",
    "requirements.txt",
    "build-requirements.txt",
    "voxcpm-source.tar",
    "compiled-source-evidence.json",
  ]) {
    const file = `build-evidence/${name}`;
    if (!manifest.files.some((entry) => entry.path === file))
      throw new Error(`Runtime build evidence is missing: ${file}`);
  }
}

function validateCompiledSources(manifest, dependencyLock) {
  const policy = join(dirname(dependencyLock), "compiled-source-evidence.json");
  const declaration = manifest.files.find(
    (file) => file.path === "build-evidence/compiled-source-evidence.json",
  );
  if (declaration.sha256 !== sha256File(policy))
    throw new Error("Native source review evidence differs from the application");
  const compiledSources = JSON.parse(readFileSync(policy, "utf8"));
  for (const source of compiledSources.sources) {
    const entry = manifest.files.find(
      (file) => file.path === `build-evidence/compiled-sources/${source.fileName}`,
    );
    if (!entry || entry.sha256 !== source.sha256 || entry.size !== source.size)
      throw new Error(`Native source archive differs: ${source.name}`);
  }
}

function validateVoxCpm(voxcpm, manifest, dependencyLock) {
  const commit = readFileSync(dependencyLock, "utf8").match(
    /https:\/\/github\.com\/OpenBMB\/VoxCPM\.git\?rev=[a-f0-9]{40}#([a-f0-9]{40})/u,
  )?.[1];
  validateVoxIdentity(voxcpm, commit);
  for (const [file, key] of [
    ["requirements.txt", "requirementsSha256"],
    ["build-requirements.txt", "buildRequirementsSha256"],
    ["voxcpm-source.tar", "sourceSha256"],
  ]) {
    const entry = manifest.files.find((item) => item.path === `build-evidence/${file}`);
    if (entry.sha256 !== voxcpm[key]) throw new Error(`Runtime producer digest differs: ${file}`);
  }
}
function validateVoxIdentity(voxcpm, commit) {
  const hashes = [
    ["tree", 40],
    ["wheelSha256", 64],
  ];
  if (
    !commit ||
    voxcpm?.commit !== commit ||
    hashes.some(([key, length]) => !new RegExp(`^[a-f0-9]{${length}}$`, "u").test(voxcpm?.[key] ?? ""))
  ) {
    throw new Error("VoxCPM build identity differs from the dependency lock");
  }
}

function validateWorkerSource(manifest, dependencyLock) {
  const source = join(dirname(dependencyLock), "src", "knovum_inference");
  const expected = listRuntimeFiles(source).filter((file) => file.endsWith(".py"));
  const prefix = "packages/knovum_inference/";
  const actual = manifest.files.filter((file) => file.path.startsWith(prefix));
  if (
    actual.length !== expected.length ||
    expected.some((path) => {
      const file = actual.find((entry) => entry.path === `${prefix}${path}`);
      return !file || file.sha256 !== sha256File(join(source, path));
    })
  )
    throw new Error("Python worker source differs from the application checkout");
}
