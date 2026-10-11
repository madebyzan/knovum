import { z } from "zod";

import { text, hash, timestamp, target, file, configuration, model, device, source } from "./contracts.mjs";
const imageCorpus = z.strictObject({
  schemaVersion: z.literal(1),
  corpusId: text,
  corpusVersion: text,
  samples: z
    .array(
      z.strictObject({
        id: text,
        original: file,
        renderedRgb: file,
        selectionMask: file,
        license: text,
        sourceUrl: z.string().regex(/^https:\/\//u),
        coverage: z
          .array(
            z.enum([
              "hair",
              "product_edges",
              "transparent",
              "text",
              "anime",
              "repeated_texture",
              "large_selection",
              "thin",
              "exif",
            ]),
          )
          .min(1),
        width: z.int().positive().max(10000),
        height: z.int().positive().max(10000),
      }),
    )
    .min(1),
  profiles: z
    .array(
      z.strictObject({
        id: text,
        capability: z.enum(["upscale", "repair", "selection", "depth"]),
        model,
        config: configuration,
        sampleIds: z.array(text).min(1),
        limits: z.strictObject({
          maximumLoadSeconds: z.number().positive(),
          maximumWarmSeconds: z.number().positive(),
          maximumPeakRssBytes: z.int().positive(),
          maximumCancelSeconds: z.number().positive(),
        }),
      }),
    )
    .min(1),
});
const imageCapture = z.strictObject({
  schemaVersion: z.literal(1),
  runId: text,
  capturedAt: timestamp,
  appCommit: z.string().regex(/^[a-f0-9]{40}$/u),
  corpusSha256: hash,
  workerDigest: hash,
  runtimeDigest: hash,
  target,
  device,
  models: z.array(
    z.strictObject({
      profileId: text,
      source,
      loadSeconds: z.number().nonnegative(),
      peakRssBytes: z.int().positive(),
      artifacts: z
        .array(
          z.strictObject({
            sampleId: text,
            output: file,
            warmSeconds: z.number().nonnegative(),
            cancelSeconds: z.number().nonnegative(),
            qualityPassed: z.boolean(),
            reviewer: text,
            reviewNotes: text,
          }),
        )
        .min(1),
    }),
  ),
});

export const imageContracts = { "image-corpus": imageCorpus, "image-capture": imageCapture };
