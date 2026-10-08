import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, copyFileSync, renameSync, rmSync, readFileSync, statSync } from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  distributionDirectory,
  readRuntimeDescriptor,
  validateRuntimeDistribution,
} from "./distribution.mjs";
import { activateDirectory, safeRelativePath, sha256File } from "./files.mjs";
import { mediaManifestName, validateMediaDescriptor, validateMediaRuntime } from "./media-manifest.mjs";
import { hostRuntimeTarget, runtimeProject, runtimeTargets } from "./profiles.mjs";
import { publicationSources } from "./publication.mjs";
import { downloadArtifact } from "./transfer.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "../..");
const { values } = parseArgs({
  options: { target: { type: "string" }, profile: { type: "string" }, publication: { type: "string" } },
});
const target = values.target ?? hostRuntimeTarget();
const profile = values.profile;
runtimeProject(repository, profile);
const supportedTargets = new Set(Object.values(runtimeTargets));
if (!supportedTargets.has(target)) throw new Error(`Unsupported bundled runtime target: ${target}`);
if (!values.publication)
  throw new Error("Provisioning requires the runtime-publication.json produced by target CI");
const publication = JSON.parse(readFileSync(values.publication, "utf8"));
const sources = publicationSources(publication, target, profile);

const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const scratch = mkdtempSync(join(repository, ".bundled-runtime-stage-"));
const destination = join(runtimeRoot, target);
try {
  const stagedRuntime = join(scratch, "runtime");
  mkdirSync(stagedRuntime);
  const downloaded = await Promise.all(
    sources.map((source) =>
      downloadArtifact(
        { url: source.url, sha256: source.expectedHash },
        join(scratch, `${source.kind}.tar`),
        { "User-Agent": "Knovum runtime provisioner" },
      ),
    ),
  );
  const archives = Object.fromEntries(sources.map((source, index) => [source.kind, downloaded[index]]));
  const sourceByKind = Object.fromEntries(sources.map((source) => [source.kind, source]));
  const distribution = join(scratch, "distribution");
  mkdirSync(distribution);
  const python = join(distribution, "runtime");
  const evidence = join(distribution, "build-evidence");
  mkdirSync(python);
  mkdirSync(evidence);
  const descriptorPath = join(distribution, "runtime-descriptor.json");
  copyFileSync(archives.PYTHON_RUNTIME_DESCRIPTOR, descriptorPath);
  const descriptor = readRuntimeDescriptor(descriptorPath, target, profile);
  if (
    descriptor.sha256 !== sourceByKind.PYTHON_RUNTIME.expectedHash ||
    descriptor.archiveUrl !== sourceByKind.PYTHON_RUNTIME.url
  )
    throw new Error("Runtime descriptor differs from the controlled archive source");
  if (statSync(archives.PYTHON_RUNTIME).size !== descriptor.size)
    throw new Error("Runtime archive size differs from its descriptor");
  for (const [kind, directory] of [
    ...(profile === "light-inference" ? [["FFMPEG", stagedRuntime]] : []),
    ["PYTHON_RUNTIME", python],
    ["PYTHON_RUNTIME_EVIDENCE", evidence],
  ]) {
    preflightArchive(archives[kind]);
    execFileSync("tar", ["-xf", archives[kind], "-C", directory], { stdio: "inherit" });
  }
  validateRuntimeDistribution(repository, target, profile, distribution);
  const distributionDestination = distributionDirectory(repository, target, profile);
  mkdirSync(dirname(distributionDestination), { recursive: true });
  rmSync(distributionDestination, { recursive: true, force: true });
  renameSync(distribution, distributionDestination);
  if (profile === "light-inference") {
    const media = JSON.parse(readFileSync(archives.FFMPEG_DESCRIPTOR, "utf8"));
    validateMediaDescriptor(repository, media, target);
    if (
      media.sha256 !== sourceByKind.FFMPEG.expectedHash ||
      media.archiveUrl !== sourceByKind.FFMPEG.url ||
      statSync(archives.FFMPEG).size !== media.size ||
      sha256File(join(stagedRuntime, mediaManifestName)) !== media.manifestSha256
    )
      throw new Error("Core media differs from its controlled publication descriptor");
    validateMediaRuntime(repository, stagedRuntime, target);
    activateDirectory(stagedRuntime, destination);
  }
  console.log(`Provisioned ${profile} runtime for ${target}`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  rmSync(scratch, { recursive: true, force: true });
}

function preflightArchive(archive) {
  const entries = execFileSync("tar", ["-tf", archive], { encoding: "utf8" }).split(/\r?\n/u).filter(Boolean);
  if (!entries.length) throw new Error("Bundled runtime archive is empty");
  const normalized = entries.map(normalizeArchiveEntry);
  if (new Set(normalized).size !== normalized.length) {
    throw new Error("Bundled runtime archive contains duplicate entries");
  }
  const verbose = execFileSync("tar", ["-tvf", archive], { encoding: "utf8" })
    .split(/\r?\n/u)
    .filter(Boolean);
  if (verbose.length !== entries.length || verbose.some((line) => !["-", "d"].includes(line[0]))) {
    throw new Error("Bundled runtime archive may contain only regular files and directories");
  }
}

function normalizeArchiveEntry(entry) {
  const path = entry.endsWith("/") ? entry.slice(0, -1) : entry;
  const containsControlCharacter = entry.includes("\0") || entry.includes("\n") || entry.includes("\r");
  if (containsControlCharacter || !safeRelativePath(path)) {
    throw new Error(`Bundled runtime archive contains an unsafe path: ${entry}`);
  }
  return path;
}
