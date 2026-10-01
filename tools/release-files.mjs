import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, mkdtempSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { decodeSigningText, httpsUrl, publicKey } from "./release-policy.mjs";

export const maxUpdateBytes = 1024 * 1024 * 1024;

export function safeFilename(value) {
  if (typeof value !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(value)) {
    throw new Error("Release assets must use safe, flat filenames");
  }
  return value;
}

export async function fileDigest(file) {
  const size = statSync(file).size;
  if (!size || size > maxUpdateBytes) throw new Error("Release asset must be between 1 byte and 1 GiB");
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
}

export function verifySignature(file, signature, key) {
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

export async function verifyReleaseFiles(directory, artifacts, key) {
  const names = new Set();
  for (const artifact of artifacts) {
    const name = safeFilename(artifact.file);
    if (names.has(name)) throw new Error("Duplicate release asset filename");
    names.add(name);
    const file = join(directory, name);
    if (await fileDigest(file) !== artifact.sha256) throw new Error(`Release asset hash mismatch: ${name}`);
    verifySignature(file, artifact.signature, key);
  }
}

export function downloadPublicFile(url, file) {
  const source = httpsUrl(url, "Public release asset").href;
  // No GitHub token is ever sent to public artifact hosts or their redirects.
  execFileSync("curl", [
    "--fail", "--silent", "--show-error", "--location", "--proto", "=https",
    "--proto-redir", "=https", "--connect-timeout", "15", "--max-time", "1800",
    "--max-filesize", String(maxUpdateBytes), "--output", file, "--", source,
  ], { stdio: "inherit" });
}
