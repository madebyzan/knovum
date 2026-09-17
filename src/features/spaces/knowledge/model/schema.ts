import { z } from "zod";

import { MAX_KNOWLEDGE_PASSAGE_BYTES } from "./passage";

const encoder = new TextEncoder();

export const knowledgeTimestampSchema = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);

const quoteSchema = z
  .object({
    exact: z.string().min(1).max(64_000),
    prefix: z.string().max(8_000).optional(),
    suffix: z.string().max(8_000).optional(),
  })
  .strict();
const rectSchema = z
  .object({
    x: z.number().finite(),
    y: z.number().finite(),
    width: z.number().finite().positive(),
    height: z.number().finite().positive(),
  })
  .strict();
const locationSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("node") }).strict(),
  z
    .object({
      kind: z.literal("media"),
      startMs: z.number().int().nonnegative(),
      endMs: z.number().int().positive(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("pdf"),
      page: z.number().int().positive(),
      rects: z.array(rectSchema).max(512).optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("word"),
      section: z.number().int().nonnegative(),
      element: z.number().int().nonnegative(),
      headingPath: z.array(z.string().max(16_000)).max(256),
      part: z
        .enum([
          "header",
          "footer",
          "first_page_header",
          "first_page_footer",
          "even_page_header",
          "even_page_footer",
        ])
        .optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("slides"),
      slide: z.number().int().positive(),
      element: z.number().int().nonnegative().optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("sheet"),
      sheet: z.string().min(1).max(16_000),
      range: z.string().max(16_000).optional(),
      drawing: z
        .object({
          xEmu: z.number().int().safe(),
          yEmu: z.number().int().safe(),
          widthEmu: z.number().int().safe().positive(),
          heightEmu: z.number().int().safe().positive(),
        })
        .strict()
        .optional(),
      element: z.number().int().nonnegative().optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("markdown"),
      heading: z.string().max(16_000).optional(),
      startOffset: z.number().int().nonnegative().optional(),
      endOffset: z.number().int().nonnegative().optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("note"),
      blockId: z.string().max(16_000).optional(),
      startOffset: z.number().int().nonnegative().optional(),
      endOffset: z.number().int().nonnegative().optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("website"),
      section: z.string().max(16_000).optional(),
      startOffset: z.number().int().nonnegative().optional(),
      endOffset: z.number().int().nonnegative().optional(),
    })
    .strict(),
  z
    .object({
      kind: z.literal("code"),
      path: z.string().max(16_000).optional(),
      lineStart: z.number().int().positive(),
      lineEnd: z.number().int().positive(),
    })
    .strict(),
]);

export const knowledgeChunkKindSchema = z.enum([
  "node",
  "text",
  "heading",
  "page",
  "slide",
  "sheet",
  "table",
  "transcript",
  "chapter",
  "code",
  "image_description",
  "video_first_frame",
]);

export const knowledgeLocatorSchema = z
  .object({
    sourceRevision: z.string().min(1).max(2_048),
    location: locationSchema,
    quote: quoteSchema.optional(),
  })
  .strict();

export const knowledgeChunkTextSchema = z
  .string()
  .min(1)
  .refine((value) => value.trim().length > 0, "Knowledge passage cannot be blank")
  .refine(
    (value) => encoder.encode(value).byteLength <= MAX_KNOWLEDGE_PASSAGE_BYTES,
    "Knowledge passage exceeds the durable byte limit",
  );

export const knowledgeChunkMetadataSchema = z
  .record(
    z.string().min(1).max(256),
    z.union([z.string().max(16_000), z.number().finite(), z.boolean(), z.null()]),
  )
  .refine(
    (value) => Object.keys(value).length <= 256,
    "Knowledge chunk metadata may contain at most 256 fields",
  );

export const knowledgeChunkSeedSchema = z
  .object({
    kind: knowledgeChunkKindSchema,
    text: knowledgeChunkTextSchema,
    locator: knowledgeLocatorSchema,
    metadata: knowledgeChunkMetadataSchema.optional(),
  })
  .strict();

export type KnowledgeLocation = z.infer<typeof locationSchema>;
export type KnowledgeLocator = z.infer<typeof knowledgeLocatorSchema>;
export type KnowledgeChunkKind = z.infer<typeof knowledgeChunkKindSchema>;
export type KnowledgeChunkSeed = z.infer<typeof knowledgeChunkSeedSchema>;
