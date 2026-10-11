import { lstatSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

import { sha256File } from "./files.mjs";
import { condaLockPath, dependencyIdentity, runtimeRecipeIdentity } from "./profiles.mjs";

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
  if (producer.recipeSha256 !== runtimeRecipeIdentity(repository, manifest.target, manifest.profile))
    throw new Error(
      "Runtime producer recipe differs from the application; rebuild the controlled publication",
    );
  if (manifest.dependencyRequirementsSha256 !== sha256File(join(evidence, "requirements.txt")))
    throw new Error("Runtime requirements differ from the producer evidence");
  validateEvidenceFiles(evidence, manifest.profile);
  if (
    JSON.stringify(JSON.parse(readFileSync(join(evidence, "conda-packages.json"), "utf8"))) !==
    JSON.stringify(producer.condaSupplier)
  )
    throw new Error("Conda package inventory differs from the producer receipt");
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
  validateRuntimeSigning(record.signing, manifest.target);
}

function validateCondaSupplier(supplier, configuration, condaPath) {
  const packages = JSON.parse(readFileSync(condaPath, "utf8"));
  if (
    supplier?.supplier !== configuration.name ||
    supplier?.micromambaVersion !== configuration.micromambaVersion ||
    supplier?.lockSha256 !== sha256File(condaPath) ||
    JSON.stringify(supplier?.packages.map(({ files: _files, ...identity }) => identity)) !==
      JSON.stringify(packages.packages)
  )
    throw new Error("Runtime producer identity differs from the controlled build");
}

export function validateRuntimeSigning(signing, target, ErrorType = Error) {
  const modes = target.startsWith("macos-") ? ["adhoc", "production"] : ["unsigned", "production"];
  if (
    !modes.includes(signing?.mode) ||
    !Number.isSafeInteger(signing?.compiledBinaryCount) ||
    signing.compiledBinaryCount < 1
  )
    throw new ErrorType("Runtime producer signing record is invalid");
}

function validateEvidenceFiles(evidence) {
  for (const name of [
    "producer.json",
    "conda-packages.json",
    "dependency-evidence.json",
    "sbom.cdx.json",
    "requirements.txt",
  ]) {
    const file = join(evidence, name);
    if (!lstatSync(file).isFile()) throw new Error(`Runtime build evidence is missing: ${file}`);
  }
}
