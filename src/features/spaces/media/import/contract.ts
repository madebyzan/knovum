import { z } from "zod";

import { spaceMediaOriginSchema } from "../origin";

export const importedMediaSchema = z
  .object({
    kind: z.enum(["video", "audio"]),
    title: z.string().trim().min(1).max(512),
    creator: z.string().trim().min(1).max(512).nullable(),
    durationSeconds: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
    intrinsicWidth: z.number().int().positive().max(100_000).nullable(),
    intrinsicHeight: z.number().int().positive().max(100_000).nullable(),
    assetPath: z.string().min(1).max(4096),
    fileName: z.string().min(1).max(1024),
    size: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    modifiedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    sourceRevision: z.string().min(1).max(128),
    previewPath: z.string().min(1).max(4096).nullable(),
    origin: spaceMediaOriginSchema,
  })
  .strict();

export type ImportedMedia = z.infer<typeof importedMediaSchema>;

export const mediaImportProgressSchema = z
  .object({
    stage: z.enum(["resolving", "downloading", "finalizing"]),
    downloadedBytes: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    totalBytes: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
    processedMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
    durationMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
  })
  .strict();

export type MediaImportProgress = z.infer<typeof mediaImportProgressSchema>;
