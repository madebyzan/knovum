import { mkdirSync, renameSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

import { validateCorpus } from "./corpus.mjs";
import { evaluateEmbedding } from "./embedding-evaluation.mjs";
import {
  EvidenceError,
  EvidenceUnavailable,
  fileDigest,
  hashedFile,
  loadEvidence,
  status,
} from "./evidence.mjs";
import { evaluateImages } from "./image-evaluation.mjs";
import { evaluateAlignment, evaluateSynthesis, evaluateTranscription } from "./speech-evaluation.mjs";

export function evaluateFiles(corpusPath, capturePath, suite = "core") {
  if (suite === "images")
    return evaluateImages(
      loadEvidence(corpusPath, "image-corpus"),
      loadEvidence(capturePath, "image-capture"),
      { corpusPath, base: dirname(capturePath), corpusBase: dirname(corpusPath) },
    );
  const corpus = loadEvidence(corpusPath, "corpus"),
    capture = loadEvidence(capturePath, "capture"),
    provenance = capture.provenance;
  validateCorpus(corpus);
  if (provenance.corpusSha256 !== fileDigest(corpusPath))
    throw new EvidenceError("Capture provenance differs from corpus bytes");
  for (const sample of [...corpus.asr.samples, ...corpus.alignment.samples])
    hashedFile(sample.audio, dirname(corpusPath));
  const evaluators = {
    embedding: evaluateEmbedding,
    asr: evaluateTranscription,
    alignment: evaluateAlignment,
    tts: evaluateSynthesis,
  };
  const capabilities = Object.fromEntries(
    Object.entries(evaluators).map(([name, evaluate]) => {
      const section = capture.capabilities[name];
      return [
        name,
        section.status === "unavailable"
          ? section
          : evaluate(corpus[name], section, dirname(capturePath), provenance),
      ];
    }),
  );
  return {
    schemaVersion: 1,
    status: status(Object.values(capabilities)),
    runId: capture.runId,
    capturedAt: capture.capturedAt,
    corpus: { id: corpus.corpusId, version: corpus.corpusVersion, sha256: provenance.corpusSha256 },
    provenance,
    capabilities,
  };
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      suite: { type: "string", default: "core" },
      corpus: { type: "string" },
      capture: { type: "string" },
      output: { type: "string" },
    },
  });
  if (!values.corpus || !values.capture || !values.output || !["core", "images"].includes(values.suite))
    throw new Error("Acceptance requires --corpus --capture --output and a valid --suite");
  let report;
  try {
    report = evaluateFiles(resolve(values.corpus), resolve(values.capture), values.suite);
  } catch (error) {
    report = {
      schemaVersion: 1,
      status: error instanceof EvidenceUnavailable ? "unavailable" : "failed",
      generatedAt: new Date().toISOString(),
      reason: error.message,
    };
  }
  const output = resolve(values.output);
  mkdirSync(dirname(output), { recursive: true });
  const temporary = `${output}.${process.pid}.tmp`;
  writeFileSync(temporary, `${JSON.stringify(report, null, 2)}\n`, { flag: "wx" });
  renameSync(temporary, output);
  process.exitCode = { passed: 0, failed: 1, unavailable: 2 }[report.status];
}
