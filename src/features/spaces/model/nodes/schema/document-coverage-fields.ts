import { z } from "zod";

import { knowledgeLocatorSchema } from "../../../knowledge/model/schema/chunk";

const resourceSchema = z
  .object({
    id: z.string().regex(/^[a-f0-9]{64}$/),
    path: z.string().min(1).max(4096),
    mediaType: z.literal("image/png"),
    width: z.number().int().positive().max(16_384),
    height: z.number().int().positive().max(16_384),
    byteSize: z
      .number()
      .int()
      .positive()
      .max(32 * 1024 * 1024),
  })
  .strict()
  .refine((resource) => resource.width * resource.height <= 24_000_000, {
    message: "Document raster exceeds 24 megapixels",
  });

const occurrenceSchema = z
  .object({
    id: z.string().min(1).max(1024),
    location: knowledgeLocatorSchema.shape.location,
    context: z.string().max(4000),
    altText: z.string().max(16_000).nullable(),
    resourceId: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    pdfInput: z
      .discriminatedUnion("kind", [
        z
          .object({
            kind: z.literal("page"),
            page: z.number().int().positive().max(2000),
            reason: z.enum(["scanned", "suspected_garbled_text", "vector_text"]),
          })
          .strict(),
        z
          .object({
            kind: z.literal("image"),
            page: z.number().int().positive().max(2000),
            rect: z
              .object({
                x: z.number().finite(),
                y: z.number().finite(),
                width: z.number().finite().positive(),
                height: z.number().finite().positive(),
              })
              .strict(),
          })
          .strict(),
      ])
      .nullable(),
    /** Why the picture could not be extracted; set exactly when it has no resource. */
    unavailable: z.string().min(1).max(16_000).nullable(),
  })
  .strict();

export const documentCoverageFieldsSchema = z
  .object({
    schemaVersion: z.literal(1),
    parser: z.enum(["pdf-inspector", "office-oxide"]),
    sourceRevision: z.string().min(1).max(128),
    pageCount: z.number().int().positive().max(2000).nullable(),
    pagesRequiringRecognition: z.array(z.number().int().positive().max(2000)).max(2000),
    pagesWithExtractedText: z.array(z.number().int().positive().max(2000)).max(2000),
    pagesWithoutExtractableContent: z.array(z.number().int().positive().max(2000)).max(2000),
    resources: z.array(resourceSchema).max(2000),
    occurrences: z.array(occurrenceSchema).max(2000),
    warnings: z
      .array(
        z.enum([
          "office_vector_drawings_unsupported",
          "spreadsheet_image_cell_anchors_unavailable",
          "spreadsheet_chart_rendering_unsupported",
          "legacy_office_visual_occurrence_coverage_limited",
          "legacy_spreadsheet_formulas_unavailable",
          "pdf_visual_coverage_limited",
        ]),
      )
      .max(16),
  })
  .strict();

export type DocumentCoverage = z.infer<typeof documentCoverageFieldsSchema>;
