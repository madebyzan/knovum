import { z } from "zod";

import { spaceMediaOriginSchema } from "../../../media/origin";
import { spaceMediaKnowledgeSchema } from "../../../media/schema";
import { spaceVisualKnowledgeSchema } from "../../../media/visual-knowledge-schema";
import { baseSpaceNodeFields } from "./base-fields";
import { documentCoverageSchema } from "./document-coverage";

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
const visualFields = {
  ...intrinsicSizeFields,
  visualKnowledge: spaceVisualKnowledgeSchema.optional(),
};

export const spaceResourceNodeSchemas = [
  z.object({ ...baseSpaceNodeFields, ...assetFields, ...visualFields, kind: z.literal("image") }).strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      ...visualFields,
      kind: z.literal("video"),
      origin: spaceMediaOriginSchema.optional(),
      creator: z.string().trim().min(1).max(512).optional(),
      durationSeconds: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
      mediaKnowledge: spaceMediaKnowledgeSchema.optional(),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      ...assetFields,
      kind: z.literal("audio"),
      origin: spaceMediaOriginSchema.optional(),
      creator: z.string().trim().min(1).max(512).optional(),
      durationSeconds: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER).optional(),
      mediaKnowledge: spaceMediaKnowledgeSchema.optional(),
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
