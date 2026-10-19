import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, mkdirSync, rmSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { run } from "@tauri-apps/cli";

import { prepareArtifacts, validatePackagedResources } from "./artifacts.mjs";
import { lockedRunnerArguments } from "./options.mjs";
import {
  platformSigningConfiguration,
  readVersion,
  updaterConfiguration,
  validateReleaseArguments,
} from "./packaging-policy.mjs";
import { copyTree } from "./resources.mjs";
import { verifyWindowsApplicationPackage } from "./windows-package-verification.mjs";

function read(repository, path) {
  return JSON.parse(readFileSync(join(repository, path), "utf8"));
}
function productionConfiguration(repository, options) {
  if (!options.production) return {};
  validateReleaseArguments(options.args);
  readVersion(repository);
  if (read(repository, "python/packaging/license-review.json").reviewStatus !== "approved")
    throw new Error("Agent dependency license review is unresolved; release packaging is blocked");
  const release = read(repository, "src-tauri/tauri.release.conf.json");
  return {
    ...release,
    bundle: { ...release.bundle, ...platformSigningConfiguration(options.target) },
    plugins: { ...release.plugins, updater: updaterConfiguration(process.env) },
  };
}
function resourceConfiguration(configuration, options) {
  const paths = {
    "resources/agent-plugins/": "resources/agent-plugins/",
    "resources/inference-runtime/": "resources/inference-runtime/",
    [`resources/bundled-runtime/${options.target}/`]: `resources/bundled-runtime/${options.target}/`,
    "resources/bundled-runtime/media-import-manifest.json":
      "resources/bundled-runtime/media-import-manifest.json",
  };
  if (options.target.startsWith("macos-")) {
    configuration.bundle = {
      ...configuration.bundle,
      macOS: {
        ...configuration.bundle?.macOS,
        ...(!options.production ? { signingIdentity: "-" } : {}),
        files: {
          "Resources/resources/agent-runtime/knovum-agent": "resources/agent-runtime/knovum-agent",
        },
      },
    };
  } else paths["resources/agent-runtime/knovum-agent/"] = "resources/agent-runtime/knovum-agent/";
  configuration.bundle = { ...configuration.bundle, resources: paths };
  return configuration;
}
function adjacentResources(repository, dependencies, options) {
  const destination = join(dirname(dependencies.worker), "resources");
  rmSync(destination, { recursive: true, force: true });
  mkdirSync(join(destination, "bundled-runtime"), { recursive: true });
  for (const path of [
    "agent-plugins",
    "inference-runtime",
    "agent-runtime/knovum-agent",
    `bundled-runtime/${options.target}`,
  ]) {
    copyTree(join(repository, "src-tauri/resources", path), join(destination, path));
  }
  copyFileSync(
    join(repository, "src-tauri/resources/bundled-runtime/media-import-manifest.json"),
    join(destination, "bundled-runtime/media-import-manifest.json"),
  );
}
export function packagingConfiguration(repository, options) {
  return resourceConfiguration(productionConfiguration(repository, options), options);
}
export async function packageDesktop(repository, options, dependencies, configuration) {
  const artifacts = prepareArtifacts(repository, options, dependencies);
  const evidence = { ...options, ...artifacts };
  validatePackagedResources(join(repository, "src-tauri/resources"), repository, evidence);
  if (!options.bundle) adjacentResources(repository, dependencies, options);
  await run(
    ["build", "--config", JSON.stringify(configuration), ...lockedRunnerArguments(options.args)],
    "pnpm package",
  );
  if (options.bundle) verifyBundles(repository, dependencies, evidence);
}
function verifyBundles(repository, dependencies, evidence) {
  const bundle = join(dirname(dependencies.worker), "bundle");
  if (evidence.target === "windows-x64") verifyWindowsBundles(repository, bundle, evidence);
  else verifyMacBundle(repository, bundle, evidence);
}
function verifyWindowsBundles(repository, bundle, evidence) {
  const formats = evidence.bundles ?? read(repository, "src-tauri/tauri.windows.conf.json").bundle.targets;
  for (const format of formats) {
    const extension = { nsis: ".exe", msi: ".msi" }[format];
    if (!extension) throw new Error(`Unsupported Windows package format: ${format}`);
    const packages = readdirSync(join(bundle, format)).filter((name) => name.endsWith(extension));
    if (!packages.length) throw new Error(`No final ${format} package was produced`);
    for (const name of packages)
      verifyWindowsApplicationPackage(join(bundle, format, name), { ...evidence, repository });
  }
}
function verifyMacBundle(repository, bundle, evidence) {
  const app = join(bundle, "macos", `${read(repository, "src-tauri/tauri.conf.json").productName}.app`);
  validatePackagedResources(join(app, "Contents/Resources/resources"), repository, evidence);
  execFileSync("codesign", ["--verify", "--deep", "--strict", app], { stdio: "inherit" });
}
