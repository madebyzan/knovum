import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { fileDigest, verifyReleaseFiles, verifySignature } from "../files.mjs";

// CI installs the official verifier. Missing minisign is a failure, never a skipped security test.
const directory = mkdtempSync(join(tmpdir(), "knovum-signature-test-"));
test.after(() => rmSync(directory, { recursive: true, force: true }));

function generateKey(name) {
  const secret = join(directory, `${name}.key`);
  const publicFile = join(directory, `${name}.pub`);
  execFileSync("minisign", ["-G", "-W", "-s", secret, "-p", publicFile], { stdio: "pipe" });
  return { secret, key: readFileSync(publicFile).toString("base64") };
}

const signer = generateKey("signer");
const other = generateKey("other");
const file = join(directory, "application.exe");
writeFileSync(file, "signed Knovum test artifact");
execFileSync("minisign", ["-S", "-s", signer.secret, "-m", file], { stdio: "pipe" });
const signature = readFileSync(`${file}.minisig`).toString("base64");

test("the official verifier accepts the approved key and exact Tauri-encoded signature", async () => {
  const artifact = { file: "application.exe", sha256: await fileDigest(file), signature };
  await verifyReleaseFiles(directory, [artifact], signer.key);
  await assert.rejects(verifyReleaseFiles(directory, [artifact, artifact], signer.key), /Duplicate/);
});

test("a valid signature made with a different key cannot be promoted", () => {
  assert.throws(() => verifySignature(file, signature, other.key));
});

test("tampering is rejected even when the manifest hash is also changed", async () => {
  const tampered = join(directory, "tampered.exe");
  writeFileSync(tampered, "attacker-controlled replacement");
  const artifact = { file: "tampered.exe", sha256: await fileDigest(tampered), signature };
  await assert.rejects(verifyReleaseFiles(directory, [artifact], signer.key));
});
