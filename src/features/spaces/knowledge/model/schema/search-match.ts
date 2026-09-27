import { z } from "zod";

import { knowledgeChunkKindSchema, knowledgeChunkMetadataSchema, knowledgeLocatorSchema } from "./chunk";

export const knowledgeSearchMatchSchema = z
  .object({
    chunkId: z.string().min(1),
    nodeId: z.string().min(1),
    nodeKind: z.string().min(1),
    nodeTitle: z.string(),
    chunkKind: knowledgeChunkKindSchema,
    metadata: knowledgeChunkMetadataSchema.optional(),
    sourceRevision: z.string().min(1),
    locator: knowledgeLocatorSchema,
    snippet: z.string(),
    text: z.string(),
    score: z.number().finite(),
    lexicalScore: z.number().finite().optional(),
    semanticScore: z.number().finite().optional(),
    spaceId: z.string().min(1).optional(),
    spaceTitle: z.string().optional(),
  })
  .strict();

export type KnowledgeSearchMatch = z.infer<typeof knowledgeSearchMatchSchema>;
