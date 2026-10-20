import { z } from "zod";

import { validateVideoComposition } from "./video-edit-constraints";
import {
  videoAssetSchema,
  videoAudioClipSchema,
  videoClipSchema,
  videoTransitionSchema,
} from "./video-edit-media";
import { videoOutputColor, videoTextOverlaySchema } from "./video-edit-text";

export const VIDEO_EDIT_PROJECT_VERSION = 1;

export const videoEditProjectSchema = z
  .object({
    version: z.literal(VIDEO_EDIT_PROJECT_VERSION),
    sourceRevision: z.string().min(1).max(128),
    assets: z.array(videoAssetSchema).min(1).max(100),
    output: z
      .object({
        width: z.number().int().min(2).max(16_384).multipleOf(2),
        height: z.number().int().min(2).max(16_384).multipleOf(2),
        frameRate: z.union([z.literal(24), z.literal(25), z.literal(30), z.literal(60)]),
        background: videoOutputColor,
      })
      .strict()
      .refine(({ width, height }) => width * height <= 33_177_600, "Video output exceeds the pixel limit"),
    videoTrack: z.object({ clips: z.array(videoClipSchema).min(1).max(500) }).strict(),
    transitions: z.array(videoTransitionSchema).max(499),
    audioTrack: z.object({ clips: z.array(videoAudioClipSchema).max(100) }).strict(),
    textTrack: z.object({ items: z.array(videoTextOverlaySchema).max(500) }).strict(),
  })
  .strict()
  .superRefine(validateVideoComposition);
