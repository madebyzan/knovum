import { execFileSync } from "node:child_process";
import {
  existsSync,
  readFileSync,
  readdirSync,
  lstatSync,
  mkdirSync,
  cpSync,
  copyFileSync,
  rmSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";

import {
  platformSigningConfiguration,
  readVersion,
  updaterConfiguration,
  validateReleaseArguments,
} from "../release/policy.mjs";
import { distributionDirectory, validateRuntimeDistribution } from "../runtime/distribution.mjs";
import { listRuntimeFiles } from "../runtime/files.mjs";
import { validatePackagedResources } from "../runtime/packaged-resources.mjs";
import { hostRuntimeTarget, runtimeProfiles, runtimeTargetFromTriple } from "../runtime/profiles.mjs";
import { verifyRuntimeSignatures } from "../runtime/signatures.mjs";
import { verifyWindowsApplicationPackage } from "../runtime/windows-package-verification.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const args = process.argv.slice(2);
const explicitRuntimeTarget = option("--runtime-target");
const buildTarget = value("--target");
const buildRuntimeTarget = runtimeTargetFromTriple(buildTarget) ?? hostRuntimeTarget();
if (explicitRuntimeTarget && explicitRuntimeTarget !== buildRuntimeTarget) {
  throw new Error(`Runtime target ${explicitRuntimeTarget} does not match build target ${buildTarget}`);
}
const target = buildRuntimeTarget;
const release = takeFlag("--release");
const releaseConfig = release ? productionConfiguration() : {};
requireRuntime(target);
const descriptorEnvironment = {};
for (const profile of runtimeProfiles) {
  const distribution = distributionDirectory(repository, target, profile);
  validateRuntimeDistribution(repository, target, profile);
  if (release) verifyRuntimeSignatures(join(distribution, "runtime"), target, repository, profile);
  const descriptor = join(repository, "src-tauri", "resources", `python-${profile}-descriptor.json`);
  copyFileSync(join(distribution, "runtime-descriptor.json"), descriptor);
  descriptorEnvironment[`KNOVUM_PYTHON_${profile.toUpperCase().replaceAll("-", "_")}_DESCRIPTOR_PATH`] =
    descriptor;
}
const worker = join(repository, "src-tauri", "resources", "python-worker");
rmSync(worker, { recursive: true, force: true });
mkdirSync(worker, { recursive: true });
cpSync(join(repository, "python", "src"), worker, {
  recursive: true,
  filter: (path) => !path.includes("__pycache__") && (lstatSync(path).isDirectory() || path.endsWith(".py")),
});
validatePackagedResources(join(repository, "src-tauri", "resources"), repository, target);

const bundleResources = [
  "resources/agent-plugins/**/*",
  "resources/bundled-runtime/README.md",
  "resources/bundled-runtime/media-import-manifest.json",
  ...runtimeResources(target),
  ...runtimeProfiles.map((profile) => `resources/python-${profile}-descriptor.json`),
  ...listRuntimeFiles(worker).map((path) => `resources/python-worker/${path}`),
];
const config = { ...releaseConfig, bundle: { ...releaseConfig.bundle, resources: bundleResources } };
// Execute the official JavaScript CLI directly: Windows cannot exec a .cmd shim without a shell.
const cli = createRequire(import.meta.url).resolve("@tauri-apps/cli/tauri.js");
execFileSync(
  process.execPath,
  [cli, "build", ...args, "--config", JSON.stringify(config), ...(release ? ["--", "--locked"] : [])],
  {
    cwd: repository,
    env: { ...process.env, ...descriptorEnvironment },
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
    if (!extension) throw new Error(`Unsupported Windows package format: ${format}`);
    const packages = readdirSync(directory).filter((name) => name.endsWith(extension));
    if (!packages.length) throw new Error(`No final ${format} package was produced`);
    for (const name of packages) {
      verifyWindowsApplicationPackage(join(directory, name), { target, repository, production: release });
    }
  }
}
if (target.startsWith("macos-")) {
  const name = readJson("src-tauri/tauri.conf.json").productName;
  const packaged = join(bundleRoot, "macos", `${name}.app`, "Contents", "Resources", "resources");
  validatePackagedResources(packaged, repository, target);
}

function productionConfiguration() {
  validateReleaseArguments(args);
  readVersion(repository);
  const kokoro = readJson("src-tauri/src/local_resources/models/kokoro-int8-multi-lang-v1_0.json");
  if (kokoro.licenseReview?.status !== "approved") {
    throw new Error("Kokoro archive license review is unresolved; release packaging is blocked");
  }
  const compiledSources = readJson("python/compiled-source-evidence.json");
  if (compiledSources.reviewStatus !== "approved") {
    throw new Error("Python native corresponding-source review is unresolved; release packaging is blocked");
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
  const required = [binary("ffmpeg", "ffmpeg.exe"), join(runtimeRoot, "media-import-manifest.json")];
  const missing = required.filter((path) => !existsSync(path));
  if (missing.length > 0) {
    throw new Error(`Bundled runtime is incomplete for ${runtimeTarget}:\n${missing.join("\n")}`);
  }
  const forbidden = [binary("yt-dlp", "yt-dlp.exe"), binary("deno", "deno.exe")].filter(existsSync);
  if (forbidden.length > 0) {
    throw new Error(`On-demand media tools must not be bundled:\n${forbidden.join("\n")}`);
  }
  if (existsSync(join(runtimeRoot, runtimeTarget, "python-runtime")))
    throw new Error("Bundled resources must exclude the on-demand Python runtime");
}

function runtimeResources(runtimeTarget) {
  const directory = join(runtimeRoot, runtimeTarget);
  return listRuntimeFiles(directory).map((path) => `resources/bundled-runtime/${runtimeTarget}/${path}`);
}

function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  const optionValue = args[index + 1];
  if (!optionValue || optionValue.startsWith("--")) throw new Error(`Missing value for ${name}`);
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
  if (!existsSync(path)) throw new Error(`Required packaging configuration is missing: ${path}`);
  return JSON.parse(readFileSync(path, "utf8"));
}
