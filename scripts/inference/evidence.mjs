import { createHash } from "node:crypto";
import { lstatSync, readFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";

import { sha256File } from "../runtime/files.mjs";
import { coreContracts } from "./core-contracts.mjs";
import { imageContracts } from "./image-contracts.mjs";

export class EvidenceError extends Error {}
export class EvidenceUnavailable extends EvidenceError {}
const evidenceContracts = { ...coreContracts, ...imageContracts };
export const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const fileDigest = sha256File;

export function loadEvidence(path, schemaName) {
  let bytes;
  try {
    const metadata = lstatSync(path);
    if (!metadata.isFile() || metadata.size > 64 * 1024 ** 2)
      throw new EvidenceError("Evidence must be a bounded regular file");
    bytes = readFileSync(path);
  } catch (error) {
    if (error.code === "ENOENT") throw new EvidenceUnavailable(`Required evidence is unavailable: ${path}`);
    throw error;
  }
  const result = evidenceContracts[schemaName].safeParse(JSON.parse(bytes));
  if (!result.success) throw new EvidenceError(`Invalid ${schemaName}: ${result.error.message}`);
  return result.data;
}

export function hashedFile(reference, base) {
  const parts = reference.path.split(/[\\/]/u);
  if (
    isAbsolute(reference.path) ||
    /^[A-Za-z]:/u.test(reference.path) ||
    parts.some((part) => !part || part === "." || part === "..")
  )
    throw new EvidenceError("Evidence path must stay below its directory");
  let current = base;
  for (const part of parts) {
    current = join(current, part);
    let metadata;
    try {
      metadata = lstatSync(current);
    } catch (error) {
      if (error.code === "ENOENT") throw new EvidenceUnavailable(`Evidence unavailable: ${current}`);
      throw error;
    }
    if (metadata.isSymbolicLink()) throw new EvidenceError("Evidence cannot traverse symbolic links");
  }
  if (!lstatSync(current).isFile() || fileDigest(current) !== reference.sha256)
    throw new EvidenceError("Evidence byte identity differs");
  return current;
}

export function execution(
  reference,
  base,
  capability,
  { provenance, kind = "bundled_worker", profile } = {},
) {
  const output = loadEvidence(hashedFile(reference, base), `${capability}-output`);
  const source = output.source;
  const method = {
    embedding: "embedding.encode",
    asr: "speech.transcribe",
    alignment: "speech.align",
    tts: "speech.synthesize",
  }[capability];
  validateInvocations(source);
  if (source.kind !== kind || source.method !== method)
    throw new EvidenceError("Execution did not use the required inference path");
  if (profile && (source.modelSpecId !== profile.modelSpecId || source.configId !== profile.configId))
    throw new EvidenceError("Execution differs from its fixed reference profile");
  if (kind === "bundled_worker") validateCandidate(source, provenance, capability);
  return output.values;
}
function validateCandidate(source, provenance, capability) {
  if (
    source.workerDigest !== provenance.workerDigest ||
    source.runtimeDigest !== provenance.runtimeDigest ||
    source.modelSpecId !== provenance.models[capability].specId ||
    source.configId !== provenance.configs[capability].id
  )
    throw new EvidenceError("Execution differs from the frozen candidate identity");
}

export function validateInvocations(source) {
  if (source.kind === "bundled_worker" && !source.workerDigest)
    throw new EvidenceError("Candidate requires a shipped worker digest");
  unique(source.invocations.map((item) => item.requestId));
  if (source.invocations.some((item) => Date.parse(item.finishedAt) < Date.parse(item.startedAt)))
    throw new EvidenceError("Invocation finishes before it starts");
}
export function unique(items) {
  if (new Set(items).size !== items.length) throw new EvidenceError("Evidence contains duplicate identities");
}
export function sameIds(actual, expected) {
  unique(actual);
  if (actual.length !== expected.length || actual.some((item) => !expected.includes(item)))
    throw new EvidenceError("Evidence identities differ from the fixed corpus");
}
export const status = (results) =>
  results.some((item) => item.status === "failed")
    ? "failed"
    : results.some((item) => item.status === "unavailable")
      ? "unavailable"
      : "passed";
