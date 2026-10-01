import { basename } from "node:path";

import { decodeBase64, httpsUrl, requireValue, stableVersion } from "./config.mjs";

export const releaseTargets = [
  "darwin-aarch64",
  "darwin-x86_64",
  "windows-aarch64",
  "windows-x86_64",
];

function signatureText(value) {
  const encoded = requireValue(value, "artifact signature");
  const text = decodeBase64(encoded, "artifact signature").toString("utf8");
  const lines = text.trimEnd().split(/\r?\n/);
  if (
    lines.length !== 4 ||
    !lines[0].startsWith("untrusted comment:") ||
    !lines[2].startsWith("trusted comment:")
  ) {
    throw new Error("Artifact signature must be a Tauri/minisign signature, not a filename or URL");
  }
  const packet = decodeBase64(lines[1], "signature packet");
  if (
    packet.length !== 74 ||
    !["Ed", "ED"].includes(packet.subarray(0, 2).toString("ascii")) ||
    decodeBase64(lines[3], "global signature").length !== 64
  ) {
    throw new Error("Invalid minisign signature envelope");
  }
  return { encoded, text };
}

export function artifactName(version, target) {
  stableVersion(version);
  if (!releaseTargets.includes(target)) throw new Error(`Unsupported release target: ${target}`);
  const extension = target.startsWith("darwin-") ? ".app.tar.gz" : "-setup.exe";
  return `Knovum_${version}_${target}${extension}`;
}

function publicationDate(value) {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?Z$/.test(value)) {
    throw new Error("Publication date must be a UTC RFC 3339 timestamp");
  }
  const normalized = value.replace(
    /(?:\.(\d{1,3}))?Z$/,
    (_, fraction) => `.${(fraction ?? "").padEnd(3, "0")}Z`,
  );
  const timestamp = Date.parse(value);
  if (!Number.isFinite(timestamp) || new Date(timestamp).toISOString() !== normalized) {
    throw new Error("Invalid publication date");
  }
  return normalized;
}

export function createManifest({ version, notes, publishedAt, baseUrl, artifacts }) {
  stableVersion(version);
  const base = new URL(httpsUrl(baseUrl, "artifact base URL").replace(/\/$/, "") + "/");
  const canonicalDate = publicationDate(publishedAt);
  requireValue(notes, "release notes");
  if (artifacts.length !== releaseTargets.length) throw new Error("The stable release matrix is incomplete");
  const platforms = {};
  for (const artifact of artifacts) {
    const name = artifactName(version, artifact.target);
    if (artifact.name !== name || basename(artifact.name) !== artifact.name) {
      throw new Error(`Unexpected artifact filename for ${artifact.target}`);
    }
    if (platforms[artifact.target]) throw new Error(`Duplicate target: ${artifact.target}`);
    platforms[artifact.target] = {
      signature: signatureText(artifact.signature).encoded,
      url: new URL(encodeURIComponent(name), base).href,
    };
  }
  return { version, notes, pub_date: canonicalDate, platforms };
}

/** Change storage location without changing the stable discovery URL or signed bytes. */
export function relocateManifest(manifest, baseUrl) {
  const { version, notes, pub_date: publishedAt, platforms } = manifest;
  if (!platforms || Object.keys(platforms).length !== releaseTargets.length) {
    throw new Error("The stable release matrix is incomplete");
  }
  const artifacts = Object.entries(platforms).map(([target, artifact]) => {
    const source = new URL(httpsUrl(artifact.url, "existing artifact URL"));
    return {
      target,
      name: decodeURIComponent(basename(source.pathname)),
      signature: artifact.signature,
    };
  });
  return createManifest({ version, notes, publishedAt, baseUrl, artifacts });
}

export { signatureText };
