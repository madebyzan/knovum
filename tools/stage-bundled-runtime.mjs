import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, mkdirSync, rmSync } from "node:fs";
import { dirname, isAbsolute, join, normalize, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const value = (name) => {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
};
const target = value("--target");
const archive = value("--archive");
const expectedSha256 = value("--sha256");
if (!target || !archive || !/^[a-f0-9]{64}$/u.test(expectedSha256 ?? "")) {
  fail("Usage: node tools/stage-bundled-runtime.mjs --target <target> --archive <archive> --sha256 <64-hex>");
}
const supportedTargets = new Set([
  "macos-arm64",
  "macos-x64",
  "windows-arm64",
  "windows-x64",
  "linux-arm64",
  "linux-x64",
]);
if (!supportedTargets.has(target)) fail(`Unsupported bundled runtime target: ${target}`);
const actual = await sha256(archive);
if (actual !== expectedSha256) {
  fail(`Bundled runtime archive SHA-256 mismatch: expected ${expectedSha256}, got ${actual}`);
}
preflightArchive(archive);
const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const destination = join(repository, "src-tauri", "resources", "bundled-runtime", target);
rmSync(destination, { recursive: true, force: true });
mkdirSync(destination, { recursive: true });
execFileSync("tar", ["-xf", resolvePath(archive), "-C", destination], { stdio: "inherit" });
console.log(`Staged verified bundled runtime archive for ${target}`);

function preflightArchive(archivePath) {
  const entries = execFileSync("tar", ["-tf", resolvePath(archivePath)], { encoding: "utf8" })
    .split(/\r?\n/u)
    .filter(Boolean);
  if (entries.length === 0) fail("Bundled runtime archive is empty");
  if (new Set(entries).size !== entries.length) fail("Bundled runtime archive contains duplicate entries");
  for (const entry of entries) {
    const normalized = normalize(entry);
    const segments = entry.replaceAll("\\", "/").split("/");
    if (
      isAbsolute(entry) ||
      /^[A-Za-z]:[\\/]/u.test(entry) ||
      segments.includes("..") ||
      normalized === ".."
    ) {
      fail(`Bundled runtime archive contains an unsafe path: ${entry}`);
    }
  }
  const verbose = execFileSync("tar", ["-tvf", resolvePath(archivePath)], { encoding: "utf8" })
    .split(/\r?\n/u)
    .filter(Boolean);
  if (verbose.length !== entries.length || verbose.some((line) => !["-", "d"].includes(line[0]))) {
    fail("Bundled runtime archive may contain only regular files and directories");
  }
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
