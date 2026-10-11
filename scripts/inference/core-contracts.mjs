import { z } from "zod";

import { text, hash, timestamp, target, file, configuration, model, device, source } from "./contracts.mjs";
const language = z.enum(["zh-CN", "en-US"]);
const scores = z.strictObject({
  pronunciation: z.number().min(1).max(5),
  naturalness: z.number().min(1).max(5),
  voiceConsistency: z.number().min(1).max(5),
  referenceSimilarity: z.number().min(1).max(5),
  emotionControl: z.number().min(1).max(5),
});
const spokenSample = z.strictObject({
  id: text,
  language,
  audio: file,
  durationMs: z.number().positive(),
  sampleRateHz: z.int().positive(),
});
const corpus = z.strictObject({
  schemaVersion: z.literal(1),
  corpusId: text,
  corpusVersion: text,
  description: text,
  embedding: z.strictObject({
    referenceProfile: z.strictObject({
      modelSpecId: text,
      configId: text,
      device: z.literal("cpu"),
      precision: z.literal("float32"),
    }),
    documents: z
      .array(z.strictObject({ id: text, kind: z.enum(["document", "note", "code", "transcript"]), text }))
      .min(1),
    queries: z.array(z.strictObject({ id: text, text, relevantDocumentIds: z.array(text).min(1) })).min(1),
  }),
  asr: z.strictObject({
    baselineProfile: z.strictObject({
      modelSpecId: text,
      configId: text,
      modelFamily: text,
      modelSize: text,
    }),
    samples: z.array(spokenSample.extend({ coverageTags: z.array(text), referenceText: z.string() })),
  }),
  alignment: z.strictObject({
    thresholds: z.strictObject({
      minimumCoverage: z.number().min(0).max(1),
      maximumMedianBoundaryErrorMs: z.number().nonnegative(),
      maximumP95BoundaryErrorMs: z.number().nonnegative(),
    }),
    samples: z.array(
      spokenSample.extend({
        units: z
          .array(
            z.strictObject({
              id: text,
              text: z.string(),
              startMs: z.number().nonnegative(),
              endMs: z.number().nonnegative(),
            }),
          )
          .min(1),
      }),
    ),
  }),
  tts: z.strictObject({
    thresholds: scores,
    samples: z.array(z.strictObject({ id: text, language, text })).min(30),
  }),
});
const unavailable = z.strictObject({ status: z.literal("unavailable"), reason: text });
const completed = z.strictObject({ status: z.literal("completed"), candidateOutput: file });
const capture = z.strictObject({
  schemaVersion: z.literal(1),
  runId: text,
  capturedAt: timestamp,
  provenance: z.strictObject({
    appVersion: text,
    appCommit: z.string().regex(/^[a-f0-9]{40}$/u),
    workerDigest: hash,
    runtimeVersion: text,
    runtimeDigest: hash,
    target,
    corpusSha256: hash,
    device,
    models: z.strictObject({ embedding: model, asr: model, alignment: model, tts: model }),
    configs: z.strictObject({
      embedding: configuration,
      asr: configuration,
      alignment: configuration,
      tts: configuration,
    }),
  }),
  capabilities: z.strictObject({
    embedding: z.discriminatedUnion("status", [
      completed.extend({ comparisonMode: z.enum(["cpu_float32", "release_profile"]), referenceOutput: file }),
      unavailable,
    ]),
    asr: z.discriminatedUnion("status", [completed.extend({ baselineOutput: file }), unavailable]),
    alignment: z.discriminatedUnion("status", [completed, unavailable]),
    tts: z.discriminatedUnion("status", [
      completed.extend({
        blinding: z.strictObject({ assignment: file, revealedAfterReviewsAt: timestamp }),
        reviews: z.array(
          z.strictObject({
            sampleId: text,
            raterId: text,
            scores,
            notes: text.nullable(),
            reviewedAt: timestamp,
          }),
        ),
      }),
      unavailable,
    ]),
  }),
});
const output = { schemaVersion: z.literal(1), source };
const vectorRecords = z.record(text, z.array(z.number()));

export const coreContracts = {
  corpus,
  capture,
  "embedding-output": z.strictObject({
    ...output,
    values: z.strictObject({ queries: vectorRecords, documents: vectorRecords }),
  }),
  "asr-output": z.strictObject({
    ...output,
    values: z.strictObject({ transcripts: z.record(text, z.string()) }),
  }),
  "alignment-output": z.strictObject({
    ...output,
    values: z.strictObject({
      samples: z.record(
        text,
        z.array(
          z.strictObject({
            id: text,
            startMs: z.number().nonnegative().nullable(),
            endMs: z.number().nonnegative().nullable(),
          }),
        ),
      ),
    }),
  }),
  "tts-output": z.strictObject({
    ...output,
    values: z.strictObject({ artifacts: z.array(z.strictObject({ sampleId: text, audio: file })) }),
  }),
};
