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
} from "../release/policy.mjs";
import { listRuntimeFiles } from "../runtime/files.mjs";
import { validatePythonRuntime } from "../runtime/manifest.mjs";
import { verifyRuntimeSignatures } from "../runtime/signatures.mjs";
import { verifyWindowsRuntimePackage } from "../runtime/windows-package-verification.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "../..");
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
if (release) verifyRuntimeSignatures(join(runtimeRoot, target, "python-runtime"), target, repository);

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
requireRuntime(target);
const bundleRoot = join(
  repository,
  "src-tauri",
  "target",
  ...(buildTarget ? [buildTarget] : []),
  "release",
  "bundle",
);
if (target === "windows-x64") {
  const formats =
    value("--bundles")?.split(",") ?? readJson("src-tauri/tauri.windows.conf.json").bundle.targets;
  for (const format of formats) {
    const directory = join(bundleRoot, format);
    const extension = { nsis: ".exe", msi: ".msi" }[format];
    if (!extension) fail(`Unsupported Windows package format: ${format}`);
    const packages = readdirSync(directory).filter((name) => name.endsWith(extension));
    if (!packages.length) fail(`No final ${format} package was produced`);
    for (const name of packages) {
      verifyWindowsRuntimePackage(join(directory, name), { target, repository, production: release });
    }
  }
}
if (target === "macos-arm64") {
  const name = readJson("src-tauri/tauri.conf.json").productName;
  const packaged = join(
    bundleRoot,
    "macos",
    `${name}.app`,
    "Contents",
    "Resources",
    "resources",
    "bundled-runtime",
    target,
    "python-runtime",
  );
  validatePythonRuntime(packaged, target, join(repository, "python", "uv.lock"));
  if (release) verifyRuntimeSignatures(packaged, target, repository);
}

function productionConfiguration() {
  validateReleaseArguments(args);
  readVersion(repository);
  const kokoro = readJson("src-tauri/src/local_resources/models/kokoro-int8-multi-lang-v1_0.json");
  if (kokoro.licenseReview?.status !== "approved") {
    fail("Kokoro archive license review is unresolved; release packaging is blocked");
  }
  const compiledSources = readJson("python/compiled-source-evidence.json");
  if (compiledSources.reviewStatus !== "approved") {
    fail("Python native corresponding-source review is unresolved; release packaging is blocked");
  }
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
    join(runtimeRoot, "media-import-manifest.json"),
    join(runtimeRoot, runtimeTarget, "python-runtime", "runtime-manifest.json"),
  ];
  const missing = required.filter((path) => !existsSync(path));
  if (missing.length > 0) {
    fail(`Bundled runtime is incomplete for ${runtimeTarget}:\n${missing.join("\n")}`);
  }
  const forbidden = [binary("yt-dlp", "yt-dlp.exe"), binary("deno", "deno.exe")].filter(existsSync);
  if (forbidden.length > 0) {
    fail(`On-demand media tools must not be bundled:\n${forbidden.join("\n")}`);
  }
  validatePythonRuntime(
    join(runtimeRoot, runtimeTarget, "python-runtime"),
    runtimeTarget,
    join(repository, "python", "uv.lock"),
  );
}

function runtimeResources(runtimeTarget) {
  const directory = join(runtimeRoot, runtimeTarget);
  return listRuntimeFiles(directory).map((path) => `resources/bundled-runtime/${runtimeTarget}/${path}`);
}

function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  const optionValue = args[index + 1];
  if (!optionValue || optionValue.startsWith("--")) fail(`Missing value for ${name}`);
  args.splice(index, 2);
  return optionValue;
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
  const hostRuntimeTarget = `${platform}-${arch}`;
  if (!["macos-arm64", "windows-x64"].includes(hostRuntimeTarget)) {
    fail(`Unsupported package target: ${hostRuntimeTarget}`);
  }
  return hostRuntimeTarget;
}

function targetFromTriple(triple) {
  if (!triple) return undefined;
  const targets = {
    "aarch64-apple-darwin": "macos-arm64",
    "x86_64-pc-windows-msvc": "windows-x64",
  };
  const resolvedTarget = targets[triple];
  if (!resolvedTarget) fail(`Unsupported package target: ${triple}`);
  return resolvedTarget;
}

function fail(message) {
  throw new Error(message);
}
