import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, createWriteStream, existsSync, mkdirSync, renameSync, rmSync } from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

import { safeRelativePath, validatePythonRuntime } from "./runtime-manifest.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const value = (name) => {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
};
const target = value("--target") ?? hostTarget();
const supportedTargets = new Set(["macos-arm64", "windows-x64"]);
if (!supportedTargets.has(target)) throw new Error(`Unsupported bundled runtime target: ${target}`);
const envKey = target.toUpperCase().replaceAll("-", "_");
const url = value("--url") ?? process.env[`KNOVUM_BUNDLED_RUNTIME_${envKey}_URL`];
const expectedSha256 = value("--sha256") ?? process.env[`KNOVUM_BUNDLED_RUNTIME_${envKey}_SHA256`];
if (!url || !/^[a-f0-9]{64}$/u.test(expectedSha256 ?? "")) {
  throw new Error(
    `Missing controlled runtime source for ${target}. Configure KNOVUM_BUNDLED_RUNTIME_${envKey}_URL and _SHA256.`,
  );
}

const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const scratch = join(repository, ".bundled-runtime-stage");
const destination = join(runtimeRoot, target);
rmSync(scratch, { recursive: true, force: true });
mkdirSync(scratch, { recursive: true });
try {
  const headers = { "User-Agent": "Knovum runtime provisioner" };
  if (process.env.KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION) {
    headers.Authorization = process.env.KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION;
  }
  const archive = await download(url, expectedSha256, join(scratch, `${target}.tar`), headers);
  preflightArchive(archive);
  const stagedRuntime = join(scratch, "runtime");
  mkdirSync(stagedRuntime);
  execFileSync("tar", ["-xf", archive, "-C", stagedRuntime], { stdio: "inherit" });
  validateRuntime(stagedRuntime);
  rmSync(destination, { recursive: true, force: true });
  renameSync(stagedRuntime, destination);
  console.log(`Provisioned bundled runtime for ${target}`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  rmSync(scratch, { recursive: true, force: true });
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
  await pipeline(Readable.fromWeb(response.body), createWriteStream(file));
  if ((await sha256(file)) !== expectedHash) {
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

function sha256(file) {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    const stream = createReadStream(file);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", reject);
    stream.on("end", () => resolve(hash.digest("hex")));
  });
}
