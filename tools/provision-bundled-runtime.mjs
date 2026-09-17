import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, createWriteStream, mkdirSync, rmSync } from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

import { provisionMediaImportRuntime } from "./provision-media-import-runtime.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const value = (name) => {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
};
const target = value("--target") ?? hostTarget();
const supportedTargets = new Set(["macos-arm64", "macos-x64", "windows-arm64", "windows-x64"]);
if (!supportedTargets.has(target)) fail(`Unsupported bundled runtime target: ${target}`);
const envKey = target.toUpperCase().replaceAll("-", "_");
const url = value("--url") ?? process.env[`KNOVUM_BUNDLED_RUNTIME_${envKey}_URL`];
const expectedSha256 = value("--sha256") ?? process.env[`KNOVUM_BUNDLED_RUNTIME_${envKey}_SHA256`];
if (!url || !/^[a-f0-9]{64}$/u.test(expectedSha256 ?? "")) {
  fail(
    `Missing controlled runtime source for ${target}. Configure KNOVUM_BUNDLED_RUNTIME_${envKey}_URL and _SHA256.`,
  );
}

const tempRoot = join(repository, ".bundled-runtime-stage");
const archive = join(tempRoot, `${target}.tar`);
rmSync(tempRoot, { recursive: true, force: true });
mkdirSync(tempRoot, { recursive: true });
try {
  const response = await fetch(url, {
    headers: process.env.KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION
      ? { Authorization: process.env.KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION }
      : undefined,
  });
  if (!response.ok || !response.body) {
    throw new Error(`Bundled runtime download failed for ${target}: HTTP ${response.status}`);
  }
  await pipeline(Readable.fromWeb(response.body), createWriteStream(archive));
  const actual = await sha256(archive);
  if (actual !== expectedSha256) {
    throw new Error(
      `Bundled runtime archive SHA-256 mismatch for ${target}: expected ${expectedSha256}, got ${actual}`,
    );
  }
  run("stage-bundled-runtime.mjs", ["--target", target, "--archive", archive, "--sha256", expectedSha256]);
  await provisionMediaImportRuntime({ repository, target });
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
}

function run(script, scriptArgs) {
  execFileSync(process.execPath, [join(repository, "tools", script), ...scriptArgs], {
    cwd: repository,
    stdio: "inherit",
  });
}

function hostTarget() {
  const platform = { darwin: "macos", win32: "windows", linux: "linux" }[process.platform];
  const arch = { arm64: "arm64", x64: "x64" }[process.arch];
  if (!platform || !arch) fail(`Unsupported bundled runtime host: ${process.platform}/${process.arch}`);
  return `${platform}-${arch}`;
}

function sha256(filePath) {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    const stream = createReadStream(filePath);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", reject);
    stream.on("end", () => resolve(hash.digest("hex")));
  });
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
