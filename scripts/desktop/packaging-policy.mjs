import { accessSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { runtimeTargets } from "./options.mjs";

function requireValue(value, name) {
  if (typeof value !== "string" || !value.trim()) throw new Error(`${name} is required`);
  return value.trim();
}

export function httpsUrl(value, name) {
  const url = new URL(requireValue(value, name));
  if (url.protocol !== "https:" || url.username || url.password || url.search || url.hash) {
    throw new Error(`${name} must be a permanent HTTPS URL without credentials, query or fragment`);
  }
  return url;
}

export function stableVersion(value) {
  if (typeof value !== "string" || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(value)) {
    throw new Error("The stable channel requires a plain major.minor.patch version");
  }
  return value;
}

function decodeSigningText(value, name) {
  const encoded = requireValue(value, name);
  const bytes = Buffer.from(encoded, "base64");
  if (bytes.toString("base64") !== encoded) throw new Error(`${name} must contain canonical base64`);
  const text = bytes.toString("utf8");
  if (!text.startsWith("untrusted comment: ")) throw new Error(`${name} is not Tauri signing output`);
  return text;
}

export function publicKey(value) {
  const encoded = requireValue(value, "KNOVUM_UPDATER_PUBKEY");
  const text = decodeSigningText(encoded, "KNOVUM_UPDATER_PUBKEY");
  const record = Buffer.from(text.split(/\r?\n/)[1] ?? "", "base64");
  if (record.length !== 42 || record.subarray(0, 2).toString("ascii") !== "Ed") {
    throw new Error("KNOVUM_UPDATER_PUBKEY must be a Tauri/minisign public key, not a path or private key");
  }
  return encoded;
}

export function readVersion(root) {
  const read = (path) => readFileSync(join(root, path), "utf8");
  if (JSON.parse(read("src-tauri/tauri.conf.json")).version !== "../package.json")
    throw new Error("Tauri must use package.json as the application version source");
  return stableVersion(JSON.parse(read("package.json")).version);
}

/**
 * Resolves the reviewed notes a release tag may publish, and owns that path so the release
 * workflow neither validates nor reads it from a second definition.
 */
export function releaseNotesPath(root, tag) {
  const version = readVersion(root);
  if (requireValue(tag, "Release tag") !== `v${version}`) throw new Error("Tag and package version differ");
  const path = join("docs", "release-notes", `${version}.md`);
  accessSync(join(root, path));
  return path;
}

export function updaterConfiguration(env) {
  requireValue(env.TAURI_SIGNING_PRIVATE_KEY, "TAURI_SIGNING_PRIVATE_KEY");
  return {
    pubkey: publicKey(env.KNOVUM_UPDATER_PUBKEY),
    endpoints: [httpsUrl(env.KNOVUM_UPDATER_ENDPOINT, "KNOVUM_UPDATER_ENDPOINT").href],
    windows: { installMode: "passive" },
  };
}

export function platformSigningConfiguration(target) {
  if (!Object.values(runtimeTargets).includes(target)) {
    throw new Error(`Unsupported signing target: ${target}`);
  }
  if (target.startsWith("macos-")) {
    return { macOS: { signingIdentity: "-" } };
  }
  return {};
}

export function validateReleaseArguments(args) {
  const seen = new Set();
  for (let index = 0; index < args.length; index += 2) {
    const name = args[index];
    if (!["--target", "--bundles", "--config"].includes(name) || seen.has(name)) {
      throw new Error(
        "Release packaging only accepts --target, --bundles and the owned release config once each",
      );
    }
    const value = requireValue(args[index + 1], name);
    if (name === "--config" && value !== "src-tauri/tauri.release.conf.json")
      throw new Error("Release packaging requires the owned release config");
    seen.add(name);
  }
}
