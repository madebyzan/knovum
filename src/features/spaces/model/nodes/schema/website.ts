import { z } from "zod";

import { baseSpaceNodeFields } from "./base-fields";
import { httpUrlSchema } from "./content";

export const spaceWebsitePayloadSchema = z
  .object({
    ...baseSpaceNodeFields,
    kind: z.literal("website"),
    url: httpUrlSchema,
    pageTitle: z.string().max(512).optional(),
    canonicalUrl: httpUrlSchema.optional(),
    siteName: z.string().max(256).optional(),
    description: z.string().max(4096).optional(),
    contentPreview: z.string().max(12_000).optional(),
    websiteSections: z
      .array(
        z
          .object({
            heading: z.string().max(512).optional(),
            text: z.string().min(1).max(12_000),
          })
          .strict(),
      )
      .max(64)
      .optional(),
    faviconPath: z.string().max(4096).optional(),
    previewImagePath: z.string().max(4096).optional(),
    metadataState: z.enum(["idle", "loading", "ready", "failed"]).optional(),
  })
  .strict();
