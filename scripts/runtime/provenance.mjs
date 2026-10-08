import { lstatSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

import { sha256File } from "./files.mjs";
import { condaLockPath, dependencyIdentity } from "./profiles.mjs";

export function validateRuntimeProvenance(directory, manifest, dependencyLock) {
  const configuration = JSON.parse(
    readFileSync(join(join(dirname(dependencyLock), "../.."), "runtime-source.json"), "utf8"),
  );
  const evidence = join(directory, "..", "build-evidence");
  const path = join(evidence, "producer.json");
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size > 16 * 1024 * 1024)
    throw new Error("Runtime producer record is invalid");
  const producer = JSON.parse(readFileSync(path, "utf8"));
  const repository = resolve(dirname(dependencyLock), "../../..");
  const condaPath = condaLockPath(repository, manifest.target, manifest.profile);
  validateProducerIdentity(producer, manifest, configuration);
  validateCondaSupplier(producer.condaSupplier, configuration.condaSupplier, condaPath);
  if (
    producer.dependencyLockSha256 !== dependencyIdentity(repository, manifest.target, manifest.profile) ||
    manifest.dependencyLockSha256 !== producer.dependencyLockSha256
  )
    throw new Error("Runtime dependency lock differs from the application");
  if (manifest.dependencyRequirementsSha256 !== sha256File(join(evidence, "requirements.txt")))
    throw new Error("Runtime requirements differ from the producer evidence");
  validateEvidenceFiles(evidence, manifest.profile);
  if (
    JSON.stringify(JSON.parse(readFileSync(join(evidence, "conda-packages.json"), "utf8"))) !==
    JSON.stringify(producer.condaSupplier)
  )
    throw new Error("Conda package inventory differs from the producer receipt");
  if (manifest.profile === "light-inference") {
    validateCompiledSources(evidence, dependencyLock);
  }
  if (manifest.profile === "voice-generation") {
    validateVoxCpm(producer.voxcpm, evidence, configuration.voiceLibrary, repository);
  }
}

function validateProducerIdentity(record, manifest, configuration) {
  const expected = {
    schemaVersion: 1,
    profile: manifest.profile,
    target: manifest.target,
    version: manifest.version,
    uvVersion: configuration.uvVersion,

    relocationVerified: true,
  };
  if (
    Object.entries(expected).some(([key, value]) => record[key] !== value) ||
    !/^[a-f0-9]{40}$/u.test(record.appCommit ?? "")
  )
    throw new Error("Runtime producer identity differs from the controlled build");
  validateSigningRecord(record.signing, manifest.target);
}

export function validateCondaSupplier(supplier, configuration, condaPath, ErrorType = Error) {
  const packages = JSON.parse(readFileSync(condaPath, "utf8"));
  if (
    supplier?.supplier !== configuration.name ||
    supplier?.micromambaVersion !== configuration.micromambaVersion ||
    supplier?.lockSha256 !== sha256File(condaPath) ||
    JSON.stringify(supplier?.packages.map(({ files: _files, ...identity }) => identity)) !==
      JSON.stringify(packages.packages)
  )
    throw new ErrorType("Runtime producer identity differs from the controlled build");
}

export function validateSigningRecord(signing, target, ErrorType = Error) {
  const modes = target.startsWith("macos-") ? ["adhoc", "production"] : ["unsigned", "production"];
  if (
    !modes.includes(signing?.mode) ||
    !Number.isSafeInteger(signing?.compiledBinaryCount) ||
    signing.compiledBinaryCount < 1
  )
    throw new ErrorType("Runtime producer signing record is invalid");
}

function validateEvidenceFiles(evidence, profile) {
  for (const name of [
    "producer.json",
    "conda-packages.json",
    "dependency-evidence.json",
    "sbom.cdx.json",
    "requirements.txt",
    ...(profile === "voice-generation"
      ? ["audio-cpp-source.tar.gz", "voice_bridge.cpp", "voice_bridge.cmake"]
      : []),
    ...(profile === "light-inference" ? ["compiled-source-evidence.json"] : []),
  ]) {
    const file = join(evidence, name);
    if (!lstatSync(file).isFile()) throw new Error(`Runtime build evidence is missing: ${file}`);
  }
}

function validateCompiledSources(evidence, dependencyLock) {
  const policy = join(dirname(dependencyLock), "../../compiled-source-evidence.json");
  if (sha256File(join(evidence, "compiled-source-evidence.json")) !== sha256File(policy))
    throw new Error("Compiled source review evidence differs from the application");
  const compiledSources = JSON.parse(readFileSync(policy, "utf8"));
  for (const source of compiledSources.sources) {
    const path = join(evidence, "compiled-sources", source.fileName);
    if (sha256File(path) !== source.sha256 || lstatSync(path).size !== source.size)
      throw new Error(`Compiled source archive differs: ${source.name}`);
  }
}

function validateVoxCpm(voiceReceipt, evidence, configuration, repository) {
  const expected = {
    name: "audio.cpp",
    version: configuration.revision,
    cmakeVersion: configuration.cmakeVersion,
    portableCpu: true,
    license: configuration.license,
    sourceSha256: configuration.source.sha256,
  };
  if (
    Object.entries(expected).some(([key, value]) => voiceReceipt?.[key] !== value) ||
    sha256File(join(evidence, "audio-cpp-source.tar.gz")) !== configuration.source.sha256
  )
    throw new Error("VoxCPM compiled source identity differs from the frozen supplier");
  for (const [file, key] of [
    ["voice_bridge.cpp", "bridgeSha256"],
    ["voice_bridge.cmake", "recipeSha256"],
  ]) {
    if (
      sha256File(join(evidence, file)) !== voiceReceipt[key] ||
      sha256File(join(repository, "scripts/runtime", file)) !== voiceReceipt[key]
    )
      throw new Error(`VoxCPM compiled adapter identity differs: ${file}`);
  }
}
