import { z } from "zod";

import { validateVideoComposition } from "./video-edit-constraints";
import { videoAssetSchema, videoAudioClipSchema, videoClipSchema, videoInterval } from "./video-edit-media";

const videoTextOverlaySchema = z
  .object({
    id: z.string().uuid(),
    ...videoInterval,
    text: z
      .string()
      .min(1)
      .max(500)
      .refine((value) => !!value.trim(), "Text cannot be blank"),
    placement: z.enum(["top", "center", "bottom"]),
    style: z
      .object({
        family: z.string().min(1).max(128),
        weight: z.number().int().min(100).max(900),
        size: z.number().int().min(16).max(96),
        color: z.string().regex(/^#[0-9A-Fa-f]{6}$/),
        preset: z.enum(["plain", "outline", "shadow", "badge"]),
      })
      .strict(),
  })
  .strict()
  .refine(({ startMs, endMs }) => endMs > startMs, "A text overlay must have a positive duration");

export const videoEditProjectSchema = z
  .object({
    version: z.literal(1),
    sourceRevision: z.string().min(1).max(128),
    assets: z.array(videoAssetSchema).min(1).max(100),
    output: z
      .object({
        width: z.number().int().min(2).max(16_384).multipleOf(2),
        height: z.number().int().min(2).max(16_384).multipleOf(2),
        frameRate: z.union([z.literal(24), z.literal(25), z.literal(30), z.literal(60)]),
      })
      .strict()
      .refine(({ width, height }) => width * height <= 33_177_600, "Video output exceeds the pixel limit"),
    videoTrack: z.object({ clips: z.array(videoClipSchema).min(1).max(500) }).strict(),
    audioTrack: z.object({ clips: z.array(videoAudioClipSchema).max(100) }).strict(),
    textTrack: z.object({ items: z.array(videoTextOverlaySchema).max(500) }).strict(),
  })
  .strict()
  .superRefine(validateVideoComposition);
