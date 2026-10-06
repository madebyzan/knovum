import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { sha256File } from "./files.mjs";
import { validatePythonRuntime } from "./manifest.mjs";
import { runtimeProject, requirementsArguments } from "./profiles.mjs";

export function distributionDirectory(repository, target, profile) {
  return join(repository, "runtime-artifacts", profile, target);
}

export function readRuntimeDescriptor(path, target, profile) {
  const value = JSON.parse(readFileSync(path, "utf8"));
  const keys = [
    "schemaVersion",
    "target",
    "version",
    "dependencyRequirementsSha256",
    "profile",
    "dependencyLockSha256",
    "sources",
    "size",
    "sha256",
    "manifestSha256",
  ];
  validateIdentity(value, target, profile, keys);
  validateArchiveSource(value);
  return value;
}

function validateIdentity(value, target, profile, keys) {
  if (
    Object.keys(value).toSorted().join() !== keys.toSorted().join() ||
    value.schemaVersion !== 2 ||
    value.profile !== profile ||
    value.target !== target ||
    value.version !== "3.11.17" ||
    [value.sha256, value.manifestSha256, value.dependencyRequirementsSha256, value.dependencyLockSha256].some(
      (hash) => !/^[a-f0-9]{64}$/u.test(hash),
    ) ||
    !Number.isSafeInteger(value.size) ||
    value.size < 1 ||
    value.size > 4 * 1024 ** 3
  )
    throw new Error(`Invalid Python runtime descriptor for ${target}`);
}

function validateArchiveSource(value) {
  if (
    Object.keys(value.sources ?? {})
      .toSorted()
      .join() !== "china,global"
  )
    throw new Error("Runtime descriptor requires global and China archive sources");
  for (const source of Object.values(value.sources)) {
    const url = new URL(source);
    if (url.protocol !== "https:" || url.username || url.password || url.hash)
      throw new Error("Runtime archive sources must be HTTPS without credentials or fragments");
  }
}

export function validateRuntimeDistribution(
  repository,
  target,
  profile,
  directory = distributionDirectory(repository, target, profile),
) {
  const descriptor = readRuntimeDescriptor(join(directory, "runtime-descriptor.json"), target, profile);
  const runtime = join(directory, "runtime");
  if (sha256File(join(runtime, "runtime-manifest.json")) !== descriptor.manifestSha256)
    throw new Error("Python runtime manifest differs from the trusted publication descriptor");
  validatePythonRuntime(runtime, target, join(runtimeProject(repository, profile), "uv.lock"));
  const manifest = JSON.parse(readFileSync(join(runtime, "runtime-manifest.json"), "utf8"));
  if (
    manifest.version !== descriptor.version ||
    manifest.profile !== descriptor.profile ||
    manifest.dependencyLockSha256 !== descriptor.dependencyLockSha256 ||
    manifest.dependencyRequirementsSha256 !== descriptor.dependencyRequirementsSha256
  )
    throw new Error("Runtime descriptor dependency identity differs from its manifest");
  const requirements = execFileSync("uv", requirementsArguments(repository, profile), {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });
  if (createHash("sha256").update(requirements).digest("hex") !== descriptor.dependencyRequirementsSha256)
    throw new Error("Published Python runtime requirements differ from the application");
  return descriptor;
}
