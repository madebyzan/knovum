import { z } from "zod";

const timelineInterval = {
  startMs: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  endMs: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
};

const videoClipSchema = z
  .object({
    id: z.string().uuid(),
    ...timelineInterval,
    volume: z.number().min(0).max(2),
  })
  .strict()
  .refine(({ startMs, endMs }) => endMs > startMs, "A video clip must have a positive duration");

const videoTextOverlaySchema = z
  .object({
    id: z.string().uuid(),
    ...timelineInterval,
    text: z.string().trim().min(1).max(500),
    placement: z.enum(["top", "center", "bottom"]),
    style: z
      .object({
        size: z.number().int().min(16).max(96),
        color: z.string().regex(/^#[0-9A-Fa-f]{6}$/),
        preset: z.enum(["plain", "outline", "shadow", "badge"]),
      })
      .strict()
      .optional(),
  })
  .strict()
  .refine(({ startMs, endMs }) => endMs > startMs, "A text overlay must have a positive duration");

export const videoEditProjectSchema = z
  .object({
    version: z.literal(1),
    sourceRevision: z.string().min(1),
    videoTrack: z.object({ clips: z.array(videoClipSchema).min(1).max(500) }).strict(),
    textTrack: z.object({ items: z.array(videoTextOverlaySchema).max(500) }).strict(),
    transform: z
      .object({
        rotation: z.union([z.literal(0), z.literal(90), z.literal(180), z.literal(270)]),
        flipHorizontal: z.boolean(),
        flipVertical: z.boolean(),
      })
      .strict(),
  })
  .strict();
