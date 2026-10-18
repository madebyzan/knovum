import { z } from "zod";

import { spaceMediaOriginSchema } from "../../schema/origin";

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
    stage: z.enum(["preparing", "resolving", "downloading", "finalizing"]),
    downloadedBytes: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    totalBytes: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
    processedMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
    durationMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
  })
  .strict();

export type MediaImportProgress = z.infer<typeof mediaImportProgressSchema>;

/** What the link turned out to be, reported before the download starts. */
export const resolvedMediaSchema = z
  .object({
    kind: z.enum(["video", "audio"]),
    title: z.string().trim().min(1).max(512),
    durationSeconds: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).nullable(),
    assetPath: z.string().min(1).max(4096),
  })
  .strict();

export type ResolvedMedia = z.infer<typeof resolvedMediaSchema>;

/** What an acquisition reports while it runs: what the source is, then how far it has got. */
export const mediaAcquisitionEventSchema = z.discriminatedUnion("event", [
  resolvedMediaSchema.extend({ event: z.literal("resolved") }),
  mediaImportProgressSchema.extend({ event: z.literal("progress") }),
]);

export type MediaAcquisitionEvent = z.infer<typeof mediaAcquisitionEventSchema>;
