import { execFileSync } from "node:child_process";
import { createWriteStream, existsSync, mkdirSync, mkdtempSync, renameSync, rmSync } from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { safeRelativePath, sha256File } from "./files.mjs";
import { validatePythonRuntime } from "./manifest.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "../..");
const { values } = parseArgs({ options: { target: { type: "string" } } });
const target = values.target ?? hostTarget();
const supportedTargets = new Set(["macos-arm64", "windows-x64"]);
if (!supportedTargets.has(target)) throw new Error(`Unsupported bundled runtime target: ${target}`);
const envKey = target.toUpperCase().replaceAll("-", "_");
const sources = ["FFMPEG", "PYTHON_RUNTIME"].map((kind) => {
  const url = process.env[`KNOVUM_${kind}_${envKey}_URL`];
  const expectedHash = process.env[`KNOVUM_${kind}_${envKey}_SHA256`];
  if (!url?.startsWith("https://") || !/^[a-f0-9]{64}$/u.test(expectedHash ?? "")) {
    throw new Error(`Missing controlled ${kind} source for ${target}; configure its HTTPS URL and SHA256.`);
  }
  return { kind, url, expectedHash };
});

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
  const archives = await Promise.all(
    sources.map((source) =>
      download(source.url, source.expectedHash, join(scratch, `${source.kind}.tar`), headers),
    ),
  );
  for (const [index, source] of sources.entries()) {
    const archive = archives[index];
    preflightArchive(archive);
    const directory =
      source.kind === "PYTHON_RUNTIME" ? join(stagedRuntime, "python-runtime") : stagedRuntime;
    if (source.kind === "PYTHON_RUNTIME") {
      if (existsSync(directory)) throw new Error("FFmpeg archive must not contain a Python runtime");
      mkdirSync(directory);
    }
    execFileSync("tar", ["-xf", archive, "-C", directory], { stdio: "inherit" });
  }
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
  console.log(`Provisioned bundled runtime for ${target}`);
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
  validatePythonRuntime(join(directory, "python-runtime"), target, join(repository, "python", "uv.lock"));
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

function hostTarget() {
  const platform = { darwin: "macos", win32: "windows" }[process.platform];
  const arch = { arm64: "arm64", x64: "x64" }[process.arch];
  if (!platform || !arch) {
    throw new Error(`Unsupported bundled runtime host: ${process.platform}/${process.arch}`);
  }
  return `${platform}-${arch}`;
}
