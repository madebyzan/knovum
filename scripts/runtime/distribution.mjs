import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { sha256File } from "./files.mjs";
import { validatePythonRuntime } from "./manifest.mjs";

export function distributionDirectory(repository, target) {
  return join(repository, "runtime-artifacts", target);
}

export function readRuntimeDescriptor(path, target) {
  const value = JSON.parse(readFileSync(path, "utf8"));
  const keys = [
    "schemaVersion",
    "target",
    "version",
    "dependencyRequirementsSha256",
    "voxcpmCommit",
    "url",
    "size",
    "sha256",
    "manifestSha256",
  ];
  validateIdentity(value, target, keys);
  validateArchiveSource(value);
  return value;
}

function validateIdentity(value, target, keys) {
  if (
    Object.keys(value).toSorted().join() !== keys.toSorted().join() ||
    value.schemaVersion !== 1 ||
    value.target !== target ||
    value.version !== "3.11.17" ||
    [value.sha256, value.manifestSha256, value.dependencyRequirementsSha256].some(
      (hash) => !/^[a-f0-9]{64}$/u.test(hash ?? ""),
    ) ||
    !/^[a-f0-9]{40}$/u.test(value.voxcpmCommit ?? "")
  )
    throw new Error(`Invalid Python runtime descriptor for ${target}`);
}

function validateArchiveSource(value) {
  const url = new URL(value.url);
  if (
    url.protocol !== "https:" ||
    url.username ||
    url.password ||
    url.hash ||
    !Number.isSafeInteger(value.size) ||
    value.size < 1 ||
    value.size > 4 * 1024 ** 3
  )
    throw new Error(`Invalid Python runtime descriptor for ${value.target}`);
}

export function validateRuntimeDistribution(
  repository,
  target,
  directory = distributionDirectory(repository, target),
) {
  const descriptor = readRuntimeDescriptor(join(directory, "runtime-descriptor.json"), target);
  const runtime = join(directory, "runtime");
  if (sha256File(join(runtime, "runtime-manifest.json")) !== descriptor.manifestSha256)
    throw new Error("Python runtime manifest differs from the trusted publication descriptor");
  validatePythonRuntime(runtime, target, join(repository, "python", "uv.lock"));
  const manifest = JSON.parse(readFileSync(join(runtime, "runtime-manifest.json"), "utf8"));
  if (
    manifest.version !== descriptor.version ||
    manifest.voxcpmCommit !== descriptor.voxcpmCommit ||
    manifest.dependencyRequirementsSha256 !== descriptor.dependencyRequirementsSha256
  )
    throw new Error("Runtime descriptor dependency identity differs from its manifest");
  const requirements = execFileSync(
    "uv",
    [
      "export",
      "--project",
      join(repository, "python"),
      "--frozen",
      "--no-emit-project",
      "--no-default-groups",
      "--no-header",
      "--no-annotate",
      "--no-emit-package",
      "voxcpm",
    ],
    { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] },
  );
  if (createHash("sha256").update(requirements).digest("hex") !== descriptor.dependencyRequirementsSha256)
    throw new Error("Published Python runtime requirements differ from the application");
  return descriptor;
}
