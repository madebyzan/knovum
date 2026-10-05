import { z } from "zod";

export const baseSpaceNodeFields = {
  title: z.string().min(1).max(512),
  tags: z.array(z.string().trim().min(1).max(80)).max(64).optional(),
  sourceRevision: z.string().min(1).max(128).optional(),
  knowledgeRevision: z.string().min(1).max(128).optional(),
};
