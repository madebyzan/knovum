import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";

import {
  platformSigningConfiguration,
  readVersion,
  updaterConfiguration,
  validateReleaseArguments,
} from "./release-policy.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const args = process.argv.slice(2);
const explicitRuntimeTarget = option("--runtime-target");
const buildTarget = value("--target");
const buildRuntimeTarget = targetFromTriple(buildTarget) ?? hostTarget();
if (explicitRuntimeTarget && explicitRuntimeTarget !== buildRuntimeTarget) {
  fail(`Runtime target ${explicitRuntimeTarget} does not match build target ${buildTarget}`);
}
const target = buildRuntimeTarget;
const release = takeFlag("--release");
const releaseConfig = release ? productionConfiguration() : {};
requireRuntime(target);

const bundleResources = [
  "resources/bundled-runtime/README.md",
  "resources/bundled-runtime/media-import-manifest.json",
  ...runtimeResources(target),
];
const config = { ...releaseConfig, bundle: { ...releaseConfig.bundle, resources: bundleResources } };
// Execute the official JavaScript CLI directly: Windows cannot exec a .cmd shim without a shell.
const cli = createRequire(import.meta.url).resolve("@tauri-apps/cli/tauri.js");
execFileSync(
  process.execPath,
  [cli, "build", ...args, "--config", JSON.stringify(config), ...(release ? ["--", "--locked"] : [])],
  {
    cwd: repository,
    stdio: "inherit",
  },
);

function productionConfiguration() {
  validateReleaseArguments(args);
  readVersion(repository);
  const production = readJson("src-tauri/tauri.release.conf.json");
  return {
    ...production,
    bundle: { ...production.bundle, ...platformSigningConfiguration(target, process.env) },
    plugins: { ...production.plugins, updater: updaterConfiguration(process.env) },
  };
}

function requireRuntime(runtimeTarget) {
  const windows = runtimeTarget.startsWith("windows-");
  const binary = (unixName, windowsName) =>
    join(runtimeRoot, runtimeTarget, windows ? windowsName : unixName);
  const required = [
    binary("ffmpeg", "ffmpeg.exe"),
    binary("whisper-cli", "whisper-cli.exe"),
    join(runtimeRoot, "media-import-manifest.json"),
  ];
  const missing = required.filter((path) => !existsSync(path));
  if (missing.length > 0) {
    fail(`Bundled runtime is incomplete for ${runtimeTarget}:\n${missing.join("\n")}`);
  }
  const forbidden = [binary("yt-dlp", "yt-dlp.exe"), binary("deno", "deno.exe")].filter(existsSync);
  if (forbidden.length > 0) {
    fail(`On-demand media tools must not be bundled:\n${forbidden.join("\n")}`);
  }
}

function runtimeResources(runtimeTarget) {
  const directory = join(runtimeRoot, runtimeTarget);
  return readdirSync(directory, { withFileTypes: true }).map((entry) => {
    if (!entry.isFile()) fail(`Bundled runtime target may contain only files: ${entry.name}`);
    return `resources/bundled-runtime/${runtimeTarget}/${entry.name}`;
  });
}

function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  const result = args[index + 1];
  if (!result || result.startsWith("--")) fail(`Missing value for ${name}`);
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
