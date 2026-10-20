import { z } from "zod";

export const videoAssetSchema = z
  .object({
    nodeId: z.string().min(1).max(128),
    sourceRevision: z.string().min(1).max(128),
    kind: z.enum(["video", "image", "audio"]),
    durationMs: z.number().int().nonnegative().max(86_400_000),
    width: z.number().int().nonnegative().max(16_384),
    height: z.number().int().nonnegative().max(16_384),
  })
  .strict()
  .refine((asset) =>
    asset.kind === "audio"
      ? asset.durationMs > 0 && asset.width === 0 && asset.height === 0
      : asset.width > 0 && asset.height > 0 && (asset.kind === "image" || asset.durationMs > 0),
  );

const videoTransformSchema = z
  .object({
    x: z.number().min(-2).max(2),
    y: z.number().min(-2).max(2),
    scale: z.number().min(0.05).max(8),
    rotation: z.number().min(-180).max(180),
    opacity: z.number().min(0).max(1),
    fit: z.enum(["contain", "cover"]),
    flipHorizontal: z.boolean(),
    flipVertical: z.boolean(),
  })
  .strict();

const sourceWindowFields = {
  id: z.string().uuid(),
  assetId: z.string().min(1).max(128),
  sourceInMs: z.number().int().nonnegative().max(86_400_000),
  sourceOutMs: z.number().int().positive().max(86_400_000),
  timelineStartMs: z.number().int().nonnegative().max(86_400_000),
  volume: z.number().min(0).max(2),
  fadeInMs: z.number().int().nonnegative(),
  fadeOutMs: z.number().int().nonnegative(),
};

export const videoClipSchema = z.object({ ...sourceWindowFields, transform: videoTransformSchema }).strict();

export const videoAudioClipSchema = z
  .object({ ...sourceWindowFields, lane: z.number().int().min(0).max(3) })
  .strict();

export const VIDEO_TRANSITION_KINDS = [
  "dissolve",
  "fadeBlack",
  "fadeWhite",
  "wipeLeft",
  "wipeRight",
  "wipeUp",
  "wipeDown",
  "slideLeft",
  "slideRight",
  "slideUp",
  "slideDown",
] as const;

export const videoTransitionSchema = z
  .object({
    id: z.string().uuid(),
    fromClipId: z.string().uuid(),
    kind: z.enum(VIDEO_TRANSITION_KINDS),
    durationMs: z.number().int().min(100).max(5_000),
  })
  .strict();
