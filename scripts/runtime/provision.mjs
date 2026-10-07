import { execFileSync } from "node:child_process";
import {
  createWriteStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  copyFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  distributionDirectory,
  readRuntimeDescriptor,
  validateRuntimeDistribution,
} from "./distribution.mjs";
import { safeRelativePath, sha256File } from "./files.mjs";
import { hostRuntimeTarget, runtimeProject, runtimeSources, runtimeTargets } from "./profiles.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "../..");
const { values } = parseArgs({ options: { target: { type: "string" }, profile: { type: "string" } } });
const target = values.target ?? hostRuntimeTarget();
const profile = values.profile;
runtimeProject(repository, profile);
const supportedTargets = new Set(Object.values(runtimeTargets));
if (!supportedTargets.has(target)) throw new Error(`Unsupported bundled runtime target: ${target}`);
const sources = runtimeSources(target, profile, process.env);

const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const scratch = mkdtempSync(join(repository, ".bundled-runtime-stage-"));
const destination = join(runtimeRoot, target);
let preserveScratch = false;
try {
  const headers = { "User-Agent": "Knovum runtime provisioner" };
  if (process.env.KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION) {
    headers.Authorization = process.env.KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION;
  }
  const stagedRuntime = join(scratch, "runtime");
  mkdirSync(stagedRuntime);
  const downloaded = await Promise.all(
    sources.map((source) =>
      download(
        source.url,
        source.expectedHash,
        join(scratch, `${source.kind}.tar`),
        source.kind === "FFMPEG" ? headers : { "User-Agent": "Knovum runtime provisioner" },
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
    !Object.values(descriptor.sources).includes(sourceByKind.PYTHON_RUNTIME.url)
  )
    throw new Error("Runtime descriptor differs from the controlled archive source");
  if ((await import("node:fs")).statSync(archives.PYTHON_RUNTIME).size !== descriptor.size)
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
    validateRuntime(stagedRuntime);
    mkdirSync(runtimeRoot, { recursive: true });
    const previous = join(scratch, "previous-runtime");
    if (existsSync(destination)) renameSync(destination, previous);
    try {
      renameSync(stagedRuntime, destination);
    } catch (error) {
      if (existsSync(previous)) {
        try {
          renameSync(previous, destination);
        } catch (restoreError) {
          preserveScratch = true;
          throw new AggregateError(
            [error, restoreError],
            `Runtime activation and restoration failed; previous runtime preserved at ${previous}`,
          );
        }
      }
      throw error;
    }
  }
  console.log(`Provisioned ${profile} runtime for ${target}`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  if (!preserveScratch) rmSync(scratch, { recursive: true, force: true });
}

function validateRuntime(directory) {
  const windows = target.startsWith("windows-");
  const required = [join(directory, windows ? "ffmpeg.exe" : "ffmpeg")];
  const missing = required.filter((path) => !existsSync(path));
  if (missing.length > 0) {
    throw new Error(`Bundled runtime is incomplete for ${target}:\n${missing.join("\n")}`);
  }
  const forbidden = [
    join(directory, windows ? "yt-dlp.exe" : "yt-dlp"),
    join(directory, windows ? "deno.exe" : "deno"),
  ].filter(existsSync);
  if (forbidden.length > 0) {
    throw new Error(
      `On-demand media tools must not be present in the bundled runtime:\n${forbidden.join("\n")}`,
    );
  }
  if (existsSync(join(directory, "python-runtime"))) throw new Error("FFmpeg archive must exclude Python");
}

async function download(
  downloadUrl,
  expectedHash,
  file,
  headers = { "User-Agent": "Knovum runtime provisioner" },
) {
  const response = await fetch(downloadUrl, { headers });
  if (!response.ok || !response.body) throw new Error(`Runtime download failed: HTTP ${response.status}`);
  if (!response.url.startsWith("https://")) throw new Error("Runtime redirect must retain HTTPS");
  await pipeline(Readable.fromWeb(response.body), createWriteStream(file));
  if (sha256File(file) !== expectedHash) {
    throw new Error(`Runtime SHA-256 mismatch: ${file}`);
  }
  return file;
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
