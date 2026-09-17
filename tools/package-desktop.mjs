import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const args = process.argv.slice(2);
const explicitRuntimeTarget = option("--runtime-target");
const buildTarget = value("--target");
const buildRuntimeTarget = targetFromTriple(buildTarget);
if (explicitRuntimeTarget && buildRuntimeTarget && explicitRuntimeTarget !== buildRuntimeTarget) {
  fail(`Runtime target ${explicitRuntimeTarget} does not match build target ${buildTarget}`);
}
const target = explicitRuntimeTarget ?? buildRuntimeTarget ?? hostTarget();
const release = takeFlag("--release");

requireRuntime(target);

const bundleResources = [
  "resources/bundled-runtime/README.md",
  "resources/bundled-runtime/media-import-manifest.json",
  "resources/bundled-runtime/fonts/**/*",
  `resources/bundled-runtime/${target}/**/*`,
];
const releaseConfig = release ? readJson("src-tauri/tauri.release.conf.json") : undefined;
const config = releaseConfig
  ? { ...releaseConfig, bundle: { ...releaseConfig.bundle, resources: bundleResources } }
  : { bundle: { resources: bundleResources } };
const executable = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
execFileSync(executable, ["exec", "tauri", "build", ...args, "--config", JSON.stringify(config)], {
  cwd: repository,
  stdio: "inherit",
});

function requireRuntime(runtimeTarget) {
  const windows = runtimeTarget.startsWith("windows-");
  const binary = (unixName, windowsName) =>
    join(runtimeRoot, runtimeTarget, windows ? windowsName : unixName);
  const required = [
    binary("ffmpeg", "ffmpeg.exe"),
    binary("yt-dlp", "yt-dlp.exe"),
    binary("deno", "deno.exe"),
    binary("whisper-cli", "whisper-cli.exe"),
    join(runtimeRoot, "media-import-manifest.json"),
    join(runtimeRoot, "fonts", "LICENSE.txt"),
    join(runtimeRoot, "fonts", "NotoSansCJKsc-Regular.otf"),
  ];
  const missing = required.filter((path) => !existsSync(path));
  if (missing.length > 0) {
    fail(`Bundled runtime is incomplete for ${runtimeTarget}:\n${missing.join("\n")}`);
  }
}

function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  const result = args[index + 1];
  args.splice(index, 2);
  return result;
}

function value(name) {
  const index = args.indexOf(name);
  return index < 0 ? undefined : args[index + 1];
}

function takeFlag(name) {
  const index = args.indexOf(name);
  if (index < 0) return false;
  args.splice(index, 1);
  return true;
}

function readJson(relativePath) {
  const path = join(repository, relativePath);
  if (!existsSync(path)) fail(`Required packaging configuration is missing: ${path}`);
  return JSON.parse(readFileSync(path, "utf8"));
}

function hostTarget() {
  const platform = { darwin: "macos", win32: "windows" }[process.platform];
  const arch = { arm64: "arm64", x64: "x64" }[process.arch];
  if (!platform || !arch) fail(`Unsupported package host: ${process.platform}/${process.arch}`);
  return `${platform}-${arch}`;
}

function targetFromTriple(triple) {
  if (!triple) return undefined;
  const targets = {
    "aarch64-apple-darwin": "macos-arm64",
    "x86_64-apple-darwin": "macos-x64",
    "aarch64-pc-windows-msvc": "windows-arm64",
    "x86_64-pc-windows-msvc": "windows-x64",
  };
  const resolvedTarget = targets[triple];
  if (!resolvedTarget) fail(`Unsupported package target: ${triple}`);
  return resolvedTarget;
}

function fail(message) {
  throw new Error(message);
}
