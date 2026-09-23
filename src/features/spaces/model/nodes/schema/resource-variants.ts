import { z } from "zod";

import { spaceMediaAnalysisSchema } from "../../../media/analysis-schema";
import { spaceMediaOriginSchema } from "../../../media/origin";
import { spaceMediaKnowledgeSchema } from "../../../media/schema";
import { spaceVisualKnowledgeSchema } from "../../../media/visual-knowledge-schema";
import { baseSpaceNodeFields } from "./base-fields";
import { documentCoverageSchema } from "./document-coverage";
import { imageEditSchema } from "./image-edit-schema";
import { videoEditProjectSchema } from "./video-edit-schema";

const assetFields = {
  assetPath: z.string().min(1).max(4096),
  fileName: z.string().min(1).max(1024),
  previewPath: z.string().max(4096).optional(),
  previewText: z
    .string()
    .max(512 * 1024)
    .optional(),
  size: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
  modifiedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
  processingState: z.enum(["importing", "processing", "failed"]).optional(),
};

const resourceEntrySchema = z
  .object({
    path: z.string().min(1).max(4096),
    kind: z.enum(["file", "folder"]),
    size: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
  })
  .strict();

const resourceEntriesSchema = z.array(resourceEntrySchema).max(200).optional();

const intrinsicSizeFields = {
  intrinsicWidth: z.number().int().positive().max(100_000).optional(),
  intrinsicHeight: z.number().int().positive().max(100_000).optional(),
};
const imageCommentCoordinate = z.number().min(0).max(1);
const imageCommentShapeSchema = z.discriminatedUnion("kind", [
  z
    .object({
      kind: z.literal("stroke"),
      points: z
        .array(imageCommentCoordinate)
        .min(4)
        .max(1024)
        .refine((points) => points.length % 2 === 0),
    })
    .strict(),
  z
    .object({
      kind: z.literal("region"),
      x: imageCommentCoordinate,
      y: imageCommentCoordinate,
      width: imageCommentCoordinate.positive(),
      height: imageCommentCoordinate.positive(),
    })
    .strict()
    .refine(({ x, y, width, height }) => x + width <= 1 && y + height <= 1),
]);
const imageCommentSchema = z
  .object({
    id: z.uuid(),
    x: imageCommentCoordinate,
    y: imageCommentCoordinate,
    shape: imageCommentShapeSchema.optional(),
    text: z.string().trim().min(1).max(2000),
    createdAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  })
  .strict();
const visualFields = {
  ...intrinsicSizeFields,
  visualKnowledge: spaceVisualKnowledgeSchema.optional(),
};
const mediaFields = {
  origin: spaceMediaOriginSchema.optional(),
  creator: z.string().trim().min(1).max(512).optional(),
  durationSeconds: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
  mediaAnalysis: spaceMediaAnalysisSchema.optional(),
  mediaKnowledge: spaceMediaKnowledgeSchema.optional(),
};

export const spaceResourceNodeSchemas = [
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      ...visualFields,
      kind: z.literal("image"),
      imageEdit: imageEditSchema.optional(),
      comments: z.array(imageCommentSchema).max(200).optional(),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      ...visualFields,
      ...mediaFields,
      kind: z.literal("video"),
      videoEditProject: videoEditProjectSchema.optional(),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      ...mediaFields,
      kind: z.literal("audio"),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      ...intrinsicSizeFields,
      kind: z.literal("file"),
      documentCoverage: documentCoverageSchema.optional(),
      extractedText: z
        .string()
        .max(10 * 1024 * 1024)
        .optional(),
      entries: resourceEntriesSchema,
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      kind: z.literal("code"),
      previewLanguage: z.string().min(1).max(64),
    })
    .strict(),
];
