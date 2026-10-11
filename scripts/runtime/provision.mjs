import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, copyFileSync, rmSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { preflightArchive } from "./archive-policy.mjs";
import {
  distributionDirectory,
  readRuntimeDescriptor,
  validateRuntimeDistribution,
} from "./distribution.mjs";
import { activateDirectory, sha256File } from "./files.mjs";
import { hostRuntimeTarget, runtimeTargets } from "./profiles.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const { values } = parseArgs({
  options: {
    target: { type: "string" },
    tag: { type: "string" },
    repository: { type: "string" },
    "source-repository": { type: "string" },
  },
});
const target = values.target ?? hostRuntimeTarget();
if (!Object.values(runtimeTargets).includes(target)) throw new Error(`Unsupported Agent target: ${target}`);
if (!/^runtime-[a-f0-9]{40}-[1-9]\d*-[1-9]\d*$/u.test(values.tag ?? ""))
  throw new Error("Select an explicit controlled runtime-<commit>-<run>-<attempt> release");
for (const name of ["repository", "source-repository"]) {
  if (!/^[A-Za-z0-9][A-Za-z0-9-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/u.test(values[name] ?? ""))
    throw new Error(`${name} must be owner/repository`);
}
const scratch = mkdtempSync(join(repository, ".agent-runtime-stage-"));
try {
  const prefix = `python-agent-${target}`;
  const files = [`${prefix}.tar.gz`, `${prefix}-evidence.tar.gz`, `${prefix}-descriptor.json`];
  execFileSync(
    "gh",
    [
      "release",
      "download",
      values.tag,
      "--repo",
      values.repository,
      ...files.flatMap((name) => ["--pattern", name]),
      "--dir",
      scratch,
    ],
    {
      stdio: "inherit",
    },
  );
  for (const name of files) {
    execFileSync(
      "gh",
      [
        "attestation",
        "verify",
        join(scratch, name),
        "--repo",
        values["source-repository"],
        "--signer-workflow",
        `${values["source-repository"]}/.github/workflows/python-runtime.yml`,
        "--source-digest",
        values.tag.split("-")[1],
        "--source-ref",
        "refs/heads/main",
        "--deny-self-hosted-runners",
      ],
      { stdio: "inherit" },
    );
  }
  const directory = join(scratch, "distribution");
  mkdirSync(directory);
  const descriptorPath = join(directory, "runtime-descriptor.json");
  copyFileSync(join(scratch, files[2]), descriptorPath);
  const descriptor = readRuntimeDescriptor(descriptorPath, target, "agent");
  const archive = join(scratch, files[0]);
  const expectedURL = `https://github.com/${values.repository}/releases/download/${values.tag}/${files[0]}`;
  if (
    descriptor.archiveUrl !== expectedURL ||
    statSync(archive).size !== descriptor.size ||
    sha256File(archive) !== descriptor.sha256
  )
    throw new Error("Agent descriptor differs from its controlled public archive");
  for (const [name, destination] of [
    [files[0], "runtime"],
    [files[1], "build-evidence"],
  ]) {
    const path = join(scratch, name);
    preflightArchive(path);
    const output = join(directory, destination);
    mkdirSync(output);
    execFileSync("tar", ["-xf", path, "-C", output], { stdio: "inherit" });
  }
  validateRuntimeDistribution(repository, target, "agent", directory);
  const destination = distributionDirectory(repository, target, "agent");
  mkdirSync(dirname(destination), { recursive: true });
  activateDirectory(directory, destination);
  console.log(`Verified controlled Agent runtime for ${target}`);
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
