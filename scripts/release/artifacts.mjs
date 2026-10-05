import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, mkdtempSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { decodeSigningText, httpsUrl, publicKey } from "./policy.mjs";

export const MAX_ARTIFACT_BYTES = 1024 * 1024 * 1024;

export function safeArtifactFilename(value) {
  if (typeof value !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(value)) {
    throw new Error("Release assets must use safe, flat filenames");
  }
  return value;
}

export async function artifactSha256(file) {
  const size = statSync(file).size;
  if (!size || size > MAX_ARTIFACT_BYTES) throw new Error("Release asset must be between 1 byte and 1 GiB");
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
}

export function verifyArtifactSignature(file, signature, key) {
  const directory = mkdtempSync(join(tmpdir(), "knovum-signature-"));
  try {
    const publicKeyFile = join(directory, "public.key");
    const signatureFile = join(directory, "artifact.minisig");
    writeFileSync(publicKeyFile, decodeSigningText(publicKey(key), "Public key"));
    writeFileSync(signatureFile, decodeSigningText(signature, "Update signature"));
    execFileSync("minisign", ["-Vm", file, "-p", publicKeyFile, "-x", signatureFile], { stdio: "pipe" });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

export async function verifyReleaseArtifacts(directory, artifacts, key) {
  const names = artifacts.map((artifact) => safeArtifactFilename(artifact.file));
  if (new Set(names).size !== names.length) throw new Error("Duplicate release asset filename");
  await Promise.all(
    artifacts.map(async (artifact) => {
      const file = join(directory, artifact.file);
      if ((await artifactSha256(file)) !== artifact.sha256) {
        throw new Error(`Release asset hash mismatch: ${artifact.file}`);
      }
      verifyArtifactSignature(file, artifact.signature, key);
    }),
  );
}

export function downloadPublicArtifact(url, file) {
  const source = httpsUrl(url, "Public release asset").href;
  // No GitHub token is ever sent to public artifact hosts or their redirects.
  execFileSync(
    "curl",
    [
      "--fail",
      "--silent",
      "--show-error",
      "--location",
      "--proto",
      "=https",
      "--proto-redir",
      "=https",
      "--connect-timeout",
      "15",
      "--max-time",
      "1800",
      "--max-filesize",
      String(MAX_ARTIFACT_BYTES),
      "--output",
      file,
      "--",
      source,
    ],
    { stdio: "inherit" },
  );
}
