import { lstatSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { sha256File } from "./files.mjs";

export function validateRuntimeProvenance(directory, manifest, dependencyLock) {
  const configuration = JSON.parse(
    readFileSync(
      join(
        manifest.profile === "assistant" ? join(dirname(dependencyLock), "../..") : dirname(dependencyLock),
        "runtime-source.json",
      ),
      "utf8",
    ),
  );
  const evidence = join(directory, "..", "build-evidence");
  const path = join(evidence, "producer.json");
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > 16 * 1024 * 1024)
    throw new Error("Runtime producer record is invalid");
  const record = JSON.parse(readFileSync(path, "utf8"));
  validateProducerIdentity(record, manifest, configuration);
  if (
    record.dependencyLockSha256 !== sha256File(dependencyLock) ||
    manifest.dependencyLockSha256 !== record.dependencyLockSha256
  )
    throw new Error("Runtime dependency lock differs from the application");
  if (manifest.dependencyRequirementsSha256 !== sha256File(join(evidence, "requirements.txt")))
    throw new Error("Runtime requirements differ from the producer evidence");
  validateEvidenceFiles(evidence, manifest.profile);
  if (manifest.profile === "inference") {
    validateCompiledSources(evidence, dependencyLock);
    validateVoxCpm(record.voxcpm, evidence, manifest, dependencyLock);
  }
}

function validateProducerIdentity(record, manifest, configuration) {
  const expected = {
    schemaVersion: 2,
    profile: manifest.profile,
    target: manifest.target,
    version: manifest.version,
    uvVersion: configuration.uvVersion,
    standaloneRelease: configuration.standaloneRelease,

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

function validateEvidenceFiles(evidence, profile) {
  for (const name of [
    "producer.json",
    "dependency-evidence.json",
    "sbom.cdx.json",
    "requirements.txt",
    ...(profile === "inference"
      ? ["build-requirements.txt", "voxcpm-source.tar", "compiled-source-evidence.json"]
      : []),
  ]) {
    const file = join(evidence, name);
    if (!lstatSync(file).isFile()) throw new Error(`Runtime build evidence is missing: ${file}`);
  }
}

function validateCompiledSources(evidence, dependencyLock) {
  const policy = join(dirname(dependencyLock), "compiled-source-evidence.json");
  if (sha256File(join(evidence, "compiled-source-evidence.json")) !== sha256File(policy))
    throw new Error("Native source review evidence differs from the application");
  const compiledSources = JSON.parse(readFileSync(policy, "utf8"));
  for (const source of compiledSources.sources) {
    const path = join(evidence, "compiled-sources", source.fileName);
    if (sha256File(path) !== source.sha256 || lstatSync(path).size !== source.size)
      throw new Error(`Native source archive differs: ${source.name}`);
  }
}

function validateVoxCpm(voxcpm, evidence, manifest, dependencyLock) {
  const commit = readFileSync(dependencyLock, "utf8").match(
    /https:\/\/github\.com\/OpenBMB\/VoxCPM\.git\?rev=[a-f0-9]{40}#([a-f0-9]{40})/u,
  )?.[1];
  validateVoxIdentity(voxcpm, commit);
  if (manifest.dependencyRequirementsSha256 !== voxcpm.requirementsSha256)
    throw new Error("Runtime requirements identity differs from the producer");
  for (const [file, key] of [
    ["requirements.txt", "requirementsSha256"],
    ["build-requirements.txt", "buildRequirementsSha256"],
    ["voxcpm-source.tar", "sourceSha256"],
  ]) {
    if (sha256File(join(evidence, file)) !== voxcpm[key])
      throw new Error(`Runtime producer digest differs: ${file}`);
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
