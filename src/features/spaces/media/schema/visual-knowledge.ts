import { z } from "zod";

export const spaceVisualKnowledgeSchema = z
  .object({
    sourceRevision: z.string().trim().min(1).max(128),
    description: z.string().trim().min(1).max(8_000),
    detectedText: z.string().trim().min(1).max(16_000).optional(),
    tags: z.array(z.string().trim().min(1).max(80)).max(32),
    generatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    provider: z.string().trim().min(1).max(128),
    model: z.string().trim().min(1).max(256),
  })
  .strict();

export type SpaceVisualKnowledge = z.infer<typeof spaceVisualKnowledgeSchema>;
