import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  createReadStream,
  createWriteStream,
  mkdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve as resolvePath } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const value = (name) => {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
};
const target = value("--target") ?? hostTarget();
const supportedTargets = new Set(["macos-arm64", "macos-x64", "windows-arm64", "windows-x64"]);
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
const manifest = JSON.parse(readFileSync(join(runtimeRoot, "media-import-manifest.json"), "utf8"));
const descriptor = manifest.targets[target];
if (!descriptor) throw new Error(`Unsupported media import runtime target: ${target}`);
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
  rmSync(destination, { recursive: true, force: true });
  mkdirSync(destination, { recursive: true });
  execFileSync("tar", ["-xf", archive, "-C", destination], { stdio: "inherit" });
  await installMediaRuntime();
  console.log(`Provisioned bundled runtime for ${target}`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  rmSync(scratch, { recursive: true, force: true });
}

async function installMediaRuntime() {
  const ytDlp = await download(
    `https://github.com/yt-dlp/yt-dlp/releases/download/${manifest.ytDlp.version}/${descriptor.ytDlp.asset}`,
    descriptor.ytDlp.sha256,
    join(scratch, descriptor.ytDlp.asset),
  );
  const denoArchive = await download(
    `https://github.com/denoland/deno/releases/download/v${manifest.deno.version}/${descriptor.deno.asset}`,
    descriptor.deno.sha256,
    join(scratch, descriptor.deno.asset),
  );
  const extracted = join(scratch, "deno");
  mkdirSync(extracted);
  execFileSync("tar", ["-xf", denoArchive, "-C", extracted]);
  const windows = target.startsWith("windows-");
  const ytDlpTarget = join(destination, windows ? "yt-dlp.exe" : "yt-dlp");
  const denoTarget = join(destination, windows ? "deno.exe" : "deno");
  const denoBinary = join(extracted, windows ? "deno.exe" : "deno");
  if ((await sha256(denoBinary)) !== descriptor.deno.binarySha256) {
    throw new Error(`Extracted Deno binary SHA-256 mismatch for ${target}`);
  }
  copyFileSync(ytDlp, ytDlpTarget);
  copyFileSync(denoBinary, denoTarget);
  if (!windows) {
    chmodSync(ytDlpTarget, 0o755);
    chmodSync(denoTarget, 0o755);
  }
  console.log(`Installed yt-dlp ${manifest.ytDlp.version} and Deno ${manifest.deno.version}`);
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
  if (new Set(entries).size !== entries.length) {
    throw new Error("Bundled runtime archive contains duplicate entries");
  }
  for (const entry of entries) {
    if (
      isAbsolute(entry) ||
      /^[A-Za-z]:[\\/]/u.test(entry) ||
      entry.replaceAll("\\", "/").split("/").includes("..")
    ) {
      throw new Error(`Bundled runtime archive contains an unsafe path: ${entry}`);
    }
  }
  const verbose = execFileSync("tar", ["-tvf", archive], { encoding: "utf8" })
    .split(/\r?\n/u)
    .filter(Boolean);
  if (verbose.length !== entries.length || verbose.some((line) => !["-", "d"].includes(line[0]))) {
    throw new Error("Bundled runtime archive may contain only regular files and directories");
  }
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
