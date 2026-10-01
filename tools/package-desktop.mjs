import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";

import { releaseConfiguration, validateReleaseArguments } from "./release/config.mjs";
import { signingConfiguration } from "./release/signing.mjs";

const repository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");
const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
const args = process.argv.slice(2);
const explicitRuntimeTarget = option("--runtime-target", true);
const buildTarget = option("--target");
const buildRuntimeTarget = targetFromTriple(buildTarget);
if (explicitRuntimeTarget && buildRuntimeTarget && explicitRuntimeTarget !== buildRuntimeTarget) {
  fail(`Runtime target ${explicitRuntimeTarget} does not match build target ${buildTarget}`);
}
const target = explicitRuntimeTarget ?? buildRuntimeTarget ?? hostTarget();
if (!/^(macos|windows)-(arm64|x64)$/.test(target)) fail(`Unsupported runtime target: ${target}`);
const release = takeFlag("--release");
const updaterConfig = release ? releaseConfiguration(process.env) : undefined;
if (release) validateReleaseArguments(args);
const signing = release ? signingConfiguration(target, process.env) : {};

requireRuntime(target);

const bundleResources = [
  "resources/bundled-runtime/README.md",
  "resources/bundled-runtime/media-import-manifest.json",
  ...runtimeResources(target),
];
const releaseConfig = updaterConfig
  ? { ...readJson("src-tauri/tauri.release.conf.json"), ...updaterConfig }
  : undefined;
const config = releaseConfig
  ? { ...releaseConfig, bundle: { ...releaseConfig.bundle, ...signing, resources: bundleResources } }
  : { bundle: { resources: bundleResources } };
const pnpmCli = process.env.npm_execpath;
if (!pnpmCli || !existsSync(pnpmCli)) fail("Run packaging through pnpm package or pnpm package:release");
execFileSync(process.execPath, [pnpmCli, "exec", "tauri", "build", ...args, "--config", JSON.stringify(config)], {
  cwd: repository,
  stdio: "inherit",
});

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

function option(name, remove = false) {
  const matches = args.flatMap((argument, index) =>
    argument === name || argument.startsWith(`${name}=`) ? [index] : [],
  );
  if (matches.length > 1) fail(`Duplicate packaging option: ${name}`);
  if (!matches.length) return undefined;
  const index = matches[0];
  const inline = args[index].startsWith(`${name}=`);
  const result = inline ? args[index].slice(name.length + 1) : args[index + 1];
  if (!result || result.startsWith("-")) fail(`Missing packaging option value: ${name}`);
  if (remove) args.splice(index, inline ? 1 : 2);
  return result;
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
