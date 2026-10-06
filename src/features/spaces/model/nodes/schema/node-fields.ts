import { z } from "zod";

import { externalContentSourceSchema } from "./external-source";

export const baseSpaceNodeFields = {
  externalSource: externalContentSourceSchema.optional(),
  title: z.string().min(1).max(512),
  tags: z.array(z.string().trim().min(1).max(80)).max(64).optional(),
  sourceRevision: z.string().min(1).max(128).optional(),
  knowledgeRevision: z.string().min(1).max(128).optional(),
};
