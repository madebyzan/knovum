import { readFileSync } from "node:fs";
import { join } from "node:path";

export const releaseTargets = {
  "aarch64-apple-darwin": "darwin-aarch64",
  "x86_64-apple-darwin": "darwin-x86_64",
  "x86_64-pc-windows-msvc": "windows-x86_64",
  "aarch64-pc-windows-msvc": "windows-aarch64",
};

export function requireValue(value, name) {
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

export function compareVersions(left, right) {
  const a = stableVersion(left).split(".").map(BigInt);
  const b = stableVersion(right).split(".").map(BigInt);
  for (let index = 0; index < a.length; index += 1) {
    if (a[index] !== b[index]) return a[index] > b[index] ? 1 : -1;
  }
  return 0;
}

export function decodeSigningText(value, name) {
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
  const version = stableVersion(JSON.parse(read("src-tauri/tauri.conf.json")).version);
  const cargo = read("src-tauri/Cargo.toml").split(/\n\[/)[0];
  const cargoVersion = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1];
  if (JSON.parse(read("package.json")).version !== version || cargoVersion !== version) {
    throw new Error("package.json, Cargo.toml and tauri.conf.json versions must match");
  }
  return version;
}

export function updaterConfiguration(env) {
  requireValue(env.TAURI_SIGNING_PRIVATE_KEY, "TAURI_SIGNING_PRIVATE_KEY");
  return {
    pubkey: publicKey(env.KNOVUM_UPDATER_PUBKEY),
    endpoints: [httpsUrl(env.KNOVUM_UPDATER_ENDPOINT, "KNOVUM_UPDATER_ENDPOINT").href],
    windows: { installMode: "passive" },
  };
}

export function distributionRepository(value) {
  const repository = requireValue(value, "KNOVUM_DISTRIBUTION_REPOSITORY");
  if (!/^[A-Za-z0-9][A-Za-z0-9-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(repository)) {
    throw new Error("KNOVUM_DISTRIBUTION_REPOSITORY must be owner/repository");
  }
  return repository;
}

export function platformSigningConfiguration(target, env) {
  if (target.startsWith("macos-")) {
    const identity = requireValue(env.APPLE_SIGNING_IDENTITY, "APPLE_SIGNING_IDENTITY");
    if (!identity.startsWith("Developer ID Application:")) {
      throw new Error("Production macOS packages require a Developer ID Application identity");
    }
    for (const name of ["APPLE_API_ISSUER", "APPLE_API_KEY", "APPLE_API_KEY_PATH"]) {
      requireValue(env[name], name);
    }
    return {};
  }
  if (!target.startsWith("windows-")) throw new Error(`Unsupported signing target: ${target}`);
  return { windows: { signCommand: requireValue(env.KNOVUM_WINDOWS_SIGN_COMMAND, "KNOVUM_WINDOWS_SIGN_COMMAND") } };
}

export function validateReleaseArguments(args) {
  const seen = new Set();
  for (let index = 0; index < args.length; index += 2) {
    const name = args[index];
    if (!["--target", "--bundles"].includes(name) || seen.has(name)) {
      throw new Error("Release packaging only accepts --target and --bundles once each");
    }
    requireValue(args[index + 1], name);
    seen.add(name);
  }
}
