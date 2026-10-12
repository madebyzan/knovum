import { z } from "zod";

export const videoInterval = {
  startMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  endMs: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
};
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
    rotation: z.union([z.literal(0), z.literal(90), z.literal(180), z.literal(270)]),
    flipHorizontal: z.boolean(),
    flipVertical: z.boolean(),
    fit: z.enum(["contain", "cover"]),
    zoom: z.number().min(1).max(4),
  })
  .strict();
const videoTransitionSchema = z
  .object({
    kind: z.enum(["dissolve", "wipeLeft", "wipeRight"]),
    durationMs: z.number().int().min(100).max(5_000),
  })
  .strict();
const clipFields = {
  id: z.string().uuid(),
  assetId: z.string().min(1).max(128),
  ...videoInterval,
  volume: z.number().min(0).max(2),
  fadeInMs: z.number().int().nonnegative(),
  fadeOutMs: z.number().int().nonnegative(),
};
export const videoClipSchema = z
  .object({
    ...clipFields,
    transform: videoTransformSchema,
    transition: videoTransitionSchema.nullable(),
  })
  .strict();
export const videoAudioClipSchema = z
  .object({
    ...clipFields,
    timelineStartMs: z.number().int().nonnegative(),
  })
  .strict();
