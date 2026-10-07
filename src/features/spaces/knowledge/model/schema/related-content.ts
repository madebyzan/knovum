import { z } from "zod";

export const relatedContentResponseSchema = z
  .object({
    state: z.enum(["ready", "unindexed", "no_content"]),
    items: z.array(
      z
        .object({
          nodeId: z.string().min(1),
          nodeKind: z.string().min(1),
          nodeTitle: z.string(),
          snippet: z.string(),
          score: z.number().finite(),
        })
        .strict(),
    ),
  })
  .strict();
