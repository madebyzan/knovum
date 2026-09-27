import { z } from "zod";

import {
  knowledgeChunkKindSchema,
  knowledgeChunkMetadataSchema,
  knowledgeChunkTextSchema,
  knowledgeLocatorSchema,
} from "../knowledge/model/schema";
import { packageTimeSchema } from "./scalar-schema";

export const chunkSchema = z
  .object({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    nodeId: z.string().min(1),
    origin: z.enum(["source", "media", "derived"]),
    kind: knowledgeChunkKindSchema,
    sourceRevision: z.string().min(1),
    text: knowledgeChunkTextSchema,
    locator: knowledgeLocatorSchema,
    metadata: knowledgeChunkMetadataSchema,
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();
