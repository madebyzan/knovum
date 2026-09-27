import { z } from "zod";

import { knowledgeChunkSeedSchema, knowledgeTimestampSchema } from "./chunk";

const knowledgeChunkOriginSchema = z.enum(["node", "source", "media", "derived"]);

const knowledgeChunkSchema = knowledgeChunkSeedSchema
  .extend({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    nodeId: z.string().min(1),
    origin: knowledgeChunkOriginSchema,
    sourceRevision: z.string().min(1),
    createdAt: knowledgeTimestampSchema,
    updatedAt: knowledgeTimestampSchema,
  })
  .strict();

export type KnowledgeChunkOrigin = z.infer<typeof knowledgeChunkOriginSchema>;
export type KnowledgeChunk = z.infer<typeof knowledgeChunkSchema>;
