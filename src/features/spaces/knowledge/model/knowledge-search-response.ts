import { z } from "zod";

import { knowledgeSearchMatchSchema } from "./knowledge-search-match";

export const knowledgeSearchResponseSchema = z
  .object({
    matches: z.array(knowledgeSearchMatchSchema),
    semantic: z.enum(["ready", "disabled", "empty"]),
  })
  .strict();

export type KnowledgeSearchResponse = z.infer<typeof knowledgeSearchResponseSchema>;
